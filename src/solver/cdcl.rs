use crate::formula::{Clause, Cnf, Literal, Model, Variable};

use self::{
    conflict::{ConflictAnalyzer, ConflictDataProvider},
    tracker::{ClauseIdx, Tracker},
};

use super::Solver;

mod conflict;
mod tracker;

#[derive(Clone, Copy)]
enum DecisionReason {
    Decision,
    UnitPropagation(ClauseIdx),
}

#[derive(Clone, Copy)]
pub struct Decision {
    decision_level: usize,
    reason: DecisionReason,
}

struct CdclDataProvider<'solver> {
    tracker: &'solver Tracker,
    decisions: &'solver Vec<Option<Decision>>,
}

impl<'solver> CdclDataProvider<'solver> {
    fn new(tracker: &'solver Tracker, decisions: &'solver Vec<Option<Decision>>) -> Self {
        CdclDataProvider { tracker, decisions }
    }

    fn decision_for_variable(&self, variable: Variable) -> &Decision {
        self.decisions[variable.index()].as_ref().unwrap()
    }
}

impl<'solver> ConflictDataProvider for CdclDataProvider<'solver> {
    fn value(&self, variable: Variable) -> bool {
        variable.partial_value(self.tracker.assignments()).unwrap()
    }

    fn level(&self, variable: Variable) -> usize {
        self.decision_for_variable(variable).decision_level
    }

    fn antecedents(&self, variable: Variable) -> Option<&Clause> {
        let decision = self.decision_for_variable(variable);

        if let DecisionReason::UnitPropagation(clause_index) = &decision.reason {
            Some(self.tracker.original_clause(*clause_index))
        } else {
            None
        }
    }
}

pub struct CdclSolver {
    /// The target formula to solve.
    formula: Cnf,
    /// A queue used in conflict analysis.
    conflict_analyzer: ConflictAnalyzer,
    /// Decision memo for each variable.
    decisions: Vec<Option<Decision>>,
    /// A history stack of decisions.
    decision_stack: Vec<Literal>,
    /// A stack tracks size of each decision level.
    /// decision_stack[frame[k-1]..frame[k]] => decisions made at level k
    frame: Vec<usize>,
    /// A data structure to efficiently track each clause's status.
    tracker: Tracker,
}

impl CdclSolver {
    fn current_level(&self) -> usize {
        self.frame.len()
    }

    fn push_decision(&mut self, literal: Literal, reason: DecisionReason) {
        if let DecisionReason::Decision = reason {
            self.frame.push(self.decision_stack.len())
        }
        self.decision_stack.push(literal);
        self.decisions[literal.index()] = Some(Decision {
            decision_level: self.current_level(),
            reason,
        });
        self.tracker.set_literal(literal);
    }

    fn pop_decision(&mut self) -> Option<(Literal, Decision)> {
        self.decision_stack.pop().map(|literal| {
            self.tracker.unset(literal.variable());
            let decision = self.decisions[literal.index()].take().unwrap();
            if let DecisionReason::Decision = decision.reason {
                self.frame.pop();
            }
            (literal, decision)
        })
    }
}

impl Solver for CdclSolver {
    fn new(formula: Cnf) -> Self {
        let tracker = Tracker::from_cnf(&formula);
        let num_variables = formula.num_variables();

        CdclSolver {
            formula,
            conflict_analyzer: ConflictAnalyzer::new(num_variables),
            decisions: vec![None; num_variables],
            decision_stack: Vec::new(),
            frame: vec![0],
            tracker,
        }
    }

    fn solve(mut self) -> Option<Model> {
        while self.tracker.satisfied_clauses().len() != self.tracker.num_clauses() {
            // Perform unit propagation
            let unit = self.tracker.unit_clauses();
            if let Some(clause_idx) = unit.iter().next().copied() {
                let literal = self.tracker.get_unit_clause_literal(clause_idx);
                self.push_decision(literal, DecisionReason::UnitPropagation(clause_idx));

                continue;
            }

            // Learn conflict clause from the first falsified clause
            if let Some(conflict_clause_index) = self.tracker.falsified_clauses().iter().next() {
                let current_level = self.current_level();

                // Panic at root means UNSAT
                if current_level == 0 {
                    return None;
                }

                let data_provider = CdclDataProvider::new(&self.tracker, &self.decisions);
                let conflicting_clause = self.tracker.original_clause(*conflict_clause_index);

                let clause_to_learn = self.conflict_analyzer.analyze(
                    &data_provider,
                    current_level,
                    conflicting_clause,
                    &self.decision_stack[*self.frame.last().unwrap()..],
                );

                let second_max = clause_to_learn
                    .iter()
                    .map(|literal| self.decisions[literal.index()].unwrap().decision_level)
                    .filter(|&level| level < current_level)
                    .max();

                let rewind_until = match second_max {
                    None => current_level - 1,
                    Some(val) => val,
                };

                self.tracker.add_clause(clause_to_learn);

                while self.current_level() > rewind_until {
                    self.pop_decision();
                }

                continue;
            }

            // Make a new decision; try the first unassigned variable.
            // TODO: implement VSIDS
            let (index, _) = self
                .tracker
                .assignments()
                .iter()
                .enumerate()
                .find(|(_index, value)| value.is_none())
                .unwrap();

            let variable = Variable::from_index(index).unwrap();
            let literal = Literal::new(variable, true);
            self.push_decision(literal, DecisionReason::Decision);
        }

        // All clauses are satisfied, fill remaining variables and return.
        let assignment = self
            .tracker
            .assignments()
            .iter()
            .map(|assign| assign.unwrap_or(true))
            .collect::<Vec<_>>();

        return Some(Model::new(self.formula, assignment));
    }
}