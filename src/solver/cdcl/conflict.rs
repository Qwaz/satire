use std::mem::replace;

use crate::formula::{Clause, Literal, Variable};

pub trait ConflictDataProvider {
    /// Returns the current value assigned to a variable.
    fn value(&self, variable: Variable) -> bool;

    /// Returns the decision level of a variable.
    fn level(&self, variable: Variable) -> usize;

    /// Returns antecedents of a variable.
    /// `None` if the variable is a decision variable.
    fn antecedents(&self, variable: Variable) -> Option<&Clause>;
}

pub struct ConflictAnalyzer {
    /// Bitmap to check if each variable is previously seen.
    seen: Vec<bool>,
    /// A queue that records seen variables.
    seen_queue: Vec<Variable>,
    /// A clause to learn
    recorded: Vec<Literal>,
    /// Unresolved variables on the current level
    unresolved_on_current_level: usize,
}

impl ConflictAnalyzer {
    pub fn new(num_variables: usize) -> Self {
        ConflictAnalyzer {
            seen: vec![false; num_variables],
            seen_queue: Vec::new(),
            recorded: Vec::new(),
            unresolved_on_current_level: 0,
        }
    }

    fn finalize(&mut self) -> Clause {
        for &var in &self.seen_queue {
            self.seen[var.index()] = false;
        }
        self.seen_queue.clear();
        let recorded = replace(&mut self.recorded, Default::default());
        self.unresolved_on_current_level = 0;

        Clause::new(recorded)
    }

    /// Mark the variable, return true if the variable is previously unseen.
    fn mark_if_unseen(&mut self, variable: Variable) -> bool {
        if self.seen[variable.index()] {
            false
        } else {
            self.seen[variable.index()] = true;
            self.seen_queue.push(variable);
            true
        }
    }

    fn add_clause<P>(&mut self, current_level: usize, data_provider: &P, clause: &Clause)
    where
        P: ConflictDataProvider,
    {
        for literal in clause.iter() {
            if self.mark_if_unseen(literal.variable()) {
                let literal_level = data_provider.level(literal.variable());
                if literal_level == current_level {
                    self.unresolved_on_current_level += 1;
                } else if literal_level != 0 {
                    self.recorded.push(literal);
                }
            }
        }
    }

    pub fn analyze<P>(
        &mut self,
        data_provider: &P,
        current_level: usize,
        conflicting_clause: &Clause,
        literals: &[Literal],
    ) -> Clause
    where
        P: ConflictDataProvider,
    {
        self.add_clause(current_level, data_provider, conflicting_clause);

        for literal in literals.iter().rev().copied() {
            let variable = literal.variable();
            if self.seen[variable.index()] {
                self.unresolved_on_current_level -= 1;
                if self.unresolved_on_current_level == 0 {
                    // First UIP reached
                    self.recorded
                        .push(Literal::new(variable, !data_provider.value(variable)));

                    return self.finalize();
                }

                // If this was not UIP, mark its antecedents
                let antecedents = data_provider.antecedents(literal.variable()).unwrap();
                self.add_clause(current_level, data_provider, antecedents);
            }
        }

        // Decision variable is guaranteed to be UIP
        unreachable!()
    }
}
