use crate::formula::{Cnf, Literal, Model, Variable};

use self::tracker::Tracker;

use super::Solver;

mod tracker;

pub struct Decision {
    level: usize,
}

pub struct CdclSolver {
    formula: Cnf,
    decision_stack: Vec<Decision>,
    tracker: Tracker,
}

impl Solver for CdclSolver {
    fn new(formula: Cnf) -> Self {
        let tracker = Tracker::from_cnf(&formula);

        CdclSolver {
            formula,
            decision_stack: Vec::new(),
            tracker,
        }
    }

    fn solve(mut self) -> Option<crate::formula::Model> {
        // TODO: This is not CDCL yet
        // but DPLL-like algorithm to test the correctness of Tracker implementation

        fn solve_inner(solver: &mut CdclSolver) -> Option<Vec<bool>> {
            if solver.tracker.satisfied_clauses().len() == solver.tracker.num_clauses() {
                // All clauses are satisfied, fill remaining variables and return.
                let assignment = solver
                    .tracker
                    .assignments()
                    .iter()
                    .map(|assign| assign.unwrap_or(true))
                    .collect::<Vec<_>>();

                return Some(assignment);
            } else if solver.tracker.falsified_clauses().len() > 0 {
                // There is a clause that can be never satisfied.
                return None;
            }

            // We need to explor emore.

            // See if there is a unit assignment.
            let unit = solver.tracker.unit_clauses();
            match unit.iter().next().copied() {
                Some(clause_idx) => {
                    let next_literal = solver.tracker.literals(clause_idx).next().unwrap();

                    solver.tracker.set_literal(next_literal);
                    if let Some(assignment) = solve_inner(solver) {
                        return Some(assignment);
                    }
                    solver.tracker.unset(next_literal.variable());

                    None
                }
                None => {
                    // Try the first unassigned variable.
                    // Note: This is an inefficient heuristics, replace with VSIDS.
                    let (index, _) = solver
                        .tracker
                        .assignments()
                        .iter()
                        .enumerate()
                        .find(|(_index, value)| value.is_none())
                        .unwrap();

                    let variable = Variable::from_index(index).unwrap();
                    let literal = Literal::new(variable, true);

                    solver.tracker.set_literal(literal);
                    if let Some(assignment) = solve_inner(solver) {
                        return Some(assignment);
                    }
                    solver.tracker.unset(variable);

                    solver.tracker.set_literal(!literal);
                    if let Some(assignment) = solve_inner(solver) {
                        return Some(assignment);
                    }
                    solver.tracker.unset(variable);

                    None
                }
            }
        }

        let assignment = solve_inner(&mut self);
        assignment.map(|assignment| Model::new(self.formula, assignment))
    }
}
