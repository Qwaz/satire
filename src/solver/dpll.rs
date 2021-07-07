use crate::formula::{Cnf, Literal, Model, Variable};

use self::inner::Watch;

use super::Solver;

/// Internal modules whose implementation details are hidden from the solver.
mod inner {
    use std::ops::{Index, IndexMut};

    use crate::formula::{Clause, Literal};

    #[derive(Debug)]
    pub struct Watch {
        positive: Vec<Vec<usize>>,
        negative: Vec<Vec<usize>>,
    }

    impl Watch {
        pub fn new(clauses: &[Clause]) -> Self {
            let mut watch = Self {
                positive: vec![Vec::new(); clauses.len()],
                negative: vec![Vec::new(); clauses.len()],
            };

            for (idx, clause) in clauses.iter().enumerate() {
                for literal in clause.iter() {
                    watch[literal].push(idx);
                }
            }

            watch
        }
    }

    impl Index<Literal> for Watch {
        type Output = Vec<usize>;

        fn index(&self, literal: Literal) -> &Self::Output {
            if literal.positive() {
                &self.positive[literal.variable().as_index()]
            } else {
                &self.negative[literal.variable().as_index()]
            }
        }
    }

    impl IndexMut<Literal> for Watch {
        fn index_mut(&mut self, literal: Literal) -> &mut Self::Output {
            if literal.positive() {
                &mut self.positive[literal.variable().as_index()]
            } else {
                &mut self.negative[literal.variable().as_index()]
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ClauseStat {
    /// Satisfied literal count in the clause.
    satisfied: usize,
    /// Unsatisfied literal count in the clause.
    unsatisfied: usize,
}

#[derive(Debug)]
pub struct DpllSolver {
    formula: Cnf,
    watch: Watch,
    /// Variable index -> assigned status
    assignment: Vec<Option<bool>>,
    /// Clause index -> clause stat
    clause_stats: Vec<ClauseStat>,
    /// Cache for `clauses.count(satisfied_literals > 0)`
    satisfied_clauses: usize,
    /// Cache for `clauses.count(unsatisfied_literals == clause.num_literals)`
    unsatisfied_clauses: usize,
    assigned_stack: Vec<Literal>,
}

impl DpllSolver {
    fn assigned_value(&self, literal: Literal) -> Option<bool> {
        let raw_assignment = self.assignment[literal.variable().as_index()];
        raw_assignment.map(|val| val ^ !literal.positive())
    }

    /// Returns a forced literal in a unit clause.
    fn forced_assignment(&self, clause_index: usize) -> Option<Literal> {
        let clause = &self.formula.clauses()[clause_index];
        let stat = &self.clause_stats[clause_index];
        if stat.satisfied == 0 && stat.unsatisfied == clause.num_literals() - 1 {
            for literal in clause.iter() {
                if self.assigned_value(literal).is_none() {
                    return Some(literal);
                }
            }
            unreachable!()
        } else {
            None
        }
    }

    /// Finds the next unit clause if exists and returns the forced literal.
    fn search_unit_clause(&self) -> Option<Literal> {
        for clause_index in 0..self.formula.clauses().len() {
            if let Some(literal) = self.forced_assignment(clause_index) {
                return Some(literal);
            }
        }

        None
    }

    fn first_unassigned(&self) -> Variable {
        let index = self
            .assignment
            .iter()
            .position(|assigned| assigned.is_none())
            .unwrap();

        Variable::from_index(index).unwrap()
    }

    fn assign_literal(&mut self, literal: Literal) {
        self.assigned_stack.push(literal);
        self.assignment[literal.variable().as_index()] = Some(literal.positive());

        for &clause_index in &self.watch[literal] {
            let mut stat = &mut self.clause_stats[clause_index];

            if stat.satisfied == 0 {
                self.satisfied_clauses += 1;
            }
            stat.satisfied += 1;
        }

        for &clause_index in &self.watch[!literal] {
            let clause = &self.formula.clauses()[clause_index];
            let mut stat = &mut self.clause_stats[clause_index];

            stat.unsatisfied += 1;
            if stat.unsatisfied == clause.num_literals() {
                self.unsatisfied_clauses += 1;
            }
        }
    }

    fn pop_assignment(&mut self) {
        let literal = self.assigned_stack.pop().unwrap();
        self.assignment[literal.variable().as_index()] = None;

        for &clause_index in &self.watch[literal] {
            let mut stat = &mut self.clause_stats[clause_index];

            if stat.satisfied == 1 {
                self.satisfied_clauses -= 1;
            }
            stat.satisfied -= 1;
        }

        for &clause_index in &self.watch[!literal] {
            let clause = &self.formula.clauses()[clause_index];
            let mut stat = &mut self.clause_stats[clause_index];

            if stat.unsatisfied == clause.num_literals() {
                self.unsatisfied_clauses -= 1;
            }
            stat.unsatisfied -= 1;
        }
    }
}

impl Solver for DpllSolver {
    fn new(formula: Cnf) -> Self {
        let num_variables = formula.num_variables();
        let num_clauses = formula.clauses().len();

        let watch = Watch::new(formula.clauses());
        let assignment = vec![None; num_variables];
        let clause_stats = vec![Default::default(); num_clauses];

        DpllSolver {
            formula,
            watch,
            assignment,
            clause_stats,
            satisfied_clauses: 0,
            unsatisfied_clauses: 0,
            assigned_stack: Vec::with_capacity(num_variables),
        }
    }

    fn solve(mut self) -> Option<Model> {
        fn solve_inner(solver: &mut DpllSolver) -> Option<Vec<bool>> {
            if solver.satisfied_clauses == solver.formula.clauses().len() {
                // All clauses are satisfied, fill remaining variables and return.
                let assignment = solver
                    .assignment
                    .iter()
                    .map(|assign| assign.unwrap_or(true))
                    .collect::<Vec<_>>();

                return Some(assignment);
            } else if solver.unsatisfied_clauses > 0 {
                // There is a clause that can be never satisfied.
                return None;
            }

            // We need to explore more.

            // See if there is a unit assignment.
            if let Some(literal) = solver.search_unit_clause() {
                solver.assign_literal(literal);
                if let Some(assignment) = solve_inner(solver) {
                    return Some(assignment);
                }
                solver.pop_assignment();

                None
            } else {
                // Try the first unassigned variable.
                // Note: This is an inefficient heuristics.
                let variable = solver.first_unassigned();
                let literal = Literal::new(variable, true);

                solver.assign_literal(literal);
                if let Some(assignment) = solve_inner(solver) {
                    return Some(assignment);
                }
                solver.pop_assignment();

                solver.assign_literal(!literal);
                if let Some(assignment) = solve_inner(solver) {
                    return Some(assignment);
                }
                solver.pop_assignment();

                None
            }
        }

        let assignment = solve_inner(&mut self);
        assignment.map(|assignment| Model::new(self.formula, assignment))
    }
}
