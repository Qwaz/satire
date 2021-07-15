use crate::formula::{Clause, Cnf, Literal, Model, Variable};

use self::{
    conflict::{ConflictAnalyzer, ConflictDataProvider},
    tracker::{ClauseIdx, Tracker},
    vsids::VsidsScoring,
};

use super::Solver;

mod conflict;
mod tracker;
mod vsids;

#[derive(Debug, Clone, Copy)]
enum DecisionReason {
    Decision,
    UnitPropagation(ClauseIdx),
}

#[derive(Debug, Clone, Copy)]
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
    /// Score tracker
    score_heuristic: VsidsScoring,
}

impl CdclSolver {
    fn current_level(&self) -> usize {
        self.frame.len()
    }

    fn push_decision(&mut self, literal: Literal, reason: DecisionReason) {
        trace!("Set {}, {:?}", literal, reason);
        if let DecisionReason::Decision = reason {
            self.frame.push(self.decision_stack.len())
        }
        self.decision_stack.push(literal);
        self.decisions[literal.index()] = Some(Decision {
            decision_level: self.current_level(),
            reason,
        });
        self.tracker.set_literal(literal);
        self.score_heuristic.remove(literal.variable());
    }

    fn pop_decision(&mut self) -> Option<(Literal, Decision)> {
        self.decision_stack.pop().map(|literal| {
            trace!("Unset {}", literal);
            self.score_heuristic.insert(literal.variable());
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
        let score_heuristic = VsidsScoring::new(&tracker);

        let num_variables = formula.num_variables();
        CdclSolver {
            formula,
            conflict_analyzer: ConflictAnalyzer::new(num_variables),
            decisions: vec![None; num_variables],
            decision_stack: Vec::new(),
            frame: Vec::new(),
            tracker,
            score_heuristic,
        }
    }

    fn solve(mut self) -> Option<Model> {
        while self.tracker.satisfied_clauses().len() != self.tracker.num_clauses() {
            // Learn conflict clause from the first falsified clause
            if let Some(conflict_clause_index) = self.tracker.falsified_clauses().iter().next() {
                let current_level = self.current_level();

                // Panic at root means UNSAT
                if current_level == 0 {
                    return None;
                }

                let data_provider = CdclDataProvider::new(&self.tracker, &self.decisions);
                let conflicting_clause = self.tracker.original_clause(*conflict_clause_index);
                trace!("Conflict {}", conflicting_clause);

                let clause_to_learn = self.conflict_analyzer.analyze(
                    &data_provider,
                    current_level,
                    conflicting_clause,
                    &self.decision_stack[*self.frame.last().unwrap()..],
                );
                trace!("Learn {}", clause_to_learn);

                let second_max = clause_to_learn
                    .iter()
                    .map(|literal| self.decisions[literal.index()].unwrap().decision_level)
                    .filter(|&level| level < current_level)
                    .max();

                let rewind_until = match second_max {
                    None => {
                        debug_assert_eq!(clause_to_learn.len(), 1);
                        0
                    }
                    Some(val) => {
                        self.score_heuristic.learn_clause(&clause_to_learn);
                        val
                    }
                };
                self.score_heuristic.decay();

                self.tracker.add_clause(clause_to_learn);

                trace!("rewind_until {}", rewind_until);
                while self.current_level() > rewind_until {
                    self.pop_decision();
                }

                continue;
            }

            let unit = self.tracker.unit_clauses();
            if let Some(clause_idx) = unit.iter().next().copied() {
                // Perform unit propagation
                let literal = self.tracker.get_unit_clause_literal(clause_idx);
                self.push_decision(literal, DecisionReason::UnitPropagation(clause_idx));
            } else {
                // Make a new decision based on VSIDS
                let variable = self.score_heuristic.top();
                let literal = Literal::new(variable, true);
                self.push_decision(literal, DecisionReason::Decision);
            }
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
