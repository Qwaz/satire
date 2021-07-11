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
}

struct Session<'inner, 'solver, P> {
    inner: &'inner mut ConflictAnalyzer,
    current_level: usize,
    data_provider: &'solver P,
    /// A clause to learn
    recorded: Vec<Literal>,
    /// Unresolved variables on the current level
    unresolved_on_current_level: usize,
}

impl<'inner, 'solver, P> Session<'inner, 'solver, P>
where
    P: ConflictDataProvider,
{
    pub fn new(
        inner: &'inner mut ConflictAnalyzer,
        data_provider: &'solver P,
        current_level: usize,
    ) -> Self {
        Session {
            inner,
            current_level,
            data_provider,
            recorded: Vec::new(),
            unresolved_on_current_level: 0,
        }
    }

    pub fn add_clause(&mut self, clause: &Clause) {
        for literal in clause.iter() {
            if self.inner.mark_if_unseen(literal.variable()) {
                let literal_level = self.data_provider.level(literal.variable());
                if literal_level == self.current_level {
                    self.unresolved_on_current_level += 1;
                } else if literal_level != 0 {
                    self.recorded.push(literal);
                }
            }
        }
    }

    pub fn seen(&self, variable: Variable) -> bool {
        self.inner.seen[variable.index()]
    }

    pub fn finish(self) -> Clause {
        self.inner.clear();
        Clause::new(self.recorded)
    }
}

impl ConflictAnalyzer {
    pub fn new(num_variables: usize) -> Self {
        ConflictAnalyzer {
            seen: vec![false; num_variables],
            seen_queue: Vec::new(),
        }
    }

    fn clear(&mut self) {
        for &var in &self.seen_queue {
            self.seen[var.index()] = false;
        }
        self.seen_queue.clear();
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
        let mut session = Session::new(self, data_provider, current_level);
        session.add_clause(conflicting_clause);

        for literal in literals.iter().rev().copied() {
            let variable = literal.variable();
            if session.seen(variable) {
                session.unresolved_on_current_level -= 1;
                if session.unresolved_on_current_level == 0 {
                    // First UIP reached
                    session
                        .recorded
                        .push(Literal::new(variable, !data_provider.value(variable)));

                    return session.finish();
                }

                // If this was not UIP, mark its antecedents
                let antecedents = data_provider.antecedents(literal.variable()).unwrap();
                session.add_clause(antecedents);
            }
        }

        // Decision variable is guaranteed to be UIP
        unreachable!()
    }
}
