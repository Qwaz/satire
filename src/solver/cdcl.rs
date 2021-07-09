use crate::formula::{Cnf, Literal, Model, Variable};

use self::tracker::{ClauseIdx, Tracker};

use super::Solver;

mod tracker;

#[derive(Clone, Copy)]
enum DecisionReason {
    FirstDecision,
    ToggledDecision,
    UnitPropagation(ClauseIdx),
}

#[derive(Clone)]
pub struct Decision {
    decision_level: usize,
    reason: DecisionReason,
}

pub struct CdclSolver {
    formula: Cnf,
    decisions: Vec<Option<Decision>>,
    decision_stack: Vec<Literal>,
    tracker: Tracker,
}

impl CdclSolver {
    fn push_decision(&mut self, literal: Literal, decision: Decision) {
        self.tracker.set_literal(literal);
        self.decision_stack.push(literal);
        self.decisions[literal.index()] = Some(decision);
    }

    fn pop_decision(&mut self) -> Option<(Literal, Decision)> {
        self.decision_stack.pop().map(|literal| {
            self.tracker.unset(literal.variable());
            let decision = self.decisions[literal.index()].take().unwrap();
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
            decisions: vec![None; num_variables],
            decision_stack: Vec::new(),
            tracker,
        }
    }

    fn solve(mut self) -> Option<Model> {
        // TODO: This is not CDCL yet
        // but DPLL-like algorithm to test the correctness of Tracker implementation

        let mut current_level = 0;
        while self.tracker.satisfied_clauses().len() != self.tracker.num_clauses() {
            // Perform unit propagation
            let unit = self.tracker.unit_clauses();
            if let Some(clause_idx) = unit.iter().next().copied() {
                let decision = Decision {
                    decision_level: current_level,
                    reason: DecisionReason::UnitPropagation(clause_idx),
                };
                let literal = self.tracker.unit_clause_literal(clause_idx);
                self.push_decision(literal, decision);

                continue;
            }

            // Backtrack if we are stuck
            if self.tracker.falsified_clauses().len() > 0 {
                // TODO: clause learning and non-chronological backtracking

                while let Some((literal, decision)) = self.pop_decision() {
                    match decision.reason {
                        DecisionReason::FirstDecision => {
                            // Flip the decision and retry
                            current_level = decision.decision_level;
                            self.push_decision(
                                !literal,
                                Decision {
                                    decision_level: current_level,
                                    reason: DecisionReason::ToggledDecision,
                                },
                            );
                            break;
                        }
                        _ => {
                            // We already popped the decision, so we are done.
                            // Continue until we find the next untoggled decision.
                        }
                    }
                }

                if self.decision_stack.is_empty() {
                    // We tried all possibilities and failed.
                    return None;
                }

                // We flipped the last untoggled decision.
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

            current_level += 1;

            let variable = Variable::from_index(index).unwrap();
            let literal = Literal::new(variable, true);
            self.push_decision(
                literal,
                Decision {
                    decision_level: current_level,
                    reason: DecisionReason::FirstDecision,
                },
            );
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
