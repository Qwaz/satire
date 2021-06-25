use crate::formula::{Cnf, Model, Variable, VariableId};

use self::inner::Watch;

/// Internal modules whose implementation details are hidden from the solver.
mod inner {
    use std::ops::{Index, IndexMut};

    use crate::formula::{Clause, Variable};

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
                for variable in clause.iter() {
                    watch[variable].push(idx);
                }
            }

            watch
        }
    }

    impl Index<Variable> for Watch {
        type Output = Vec<usize>;

        fn index(&self, variable: Variable) -> &Self::Output {
            if variable.positive() {
                &self.positive[variable.id().as_index()]
            } else {
                &self.negative[variable.id().as_index()]
            }
        }
    }

    impl IndexMut<Variable> for Watch {
        fn index_mut(&mut self, variable: Variable) -> &mut Self::Output {
            if variable.positive() {
                &mut self.positive[variable.id().as_index()]
            } else {
                &mut self.negative[variable.id().as_index()]
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ClauseStat {
    /// Satisfied variables in the clause.
    satisfied: usize,
    /// Unsatisfied variables in the clause.
    unsatisfied: usize,
}

#[derive(Debug)]
struct DpllSolver {
    formula: Cnf,
    watch: Watch,
    /// Variable index -> assigned status
    assignment: Vec<Option<bool>>,
    /// Clause index -> clause stat
    clause_stats: Vec<ClauseStat>,
    /// Cache for `clauses.count(satisfied_variables > 0)`
    satisfied_clauses: usize,
    /// Cache for `clauses.count(unsatisfied_variables == clause.num_variables)`
    unsatisfied_clauses: usize,
    assigned_stack: Vec<Variable>,
}

impl DpllSolver {
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

    fn assigned_value(&self, variable: Variable) -> Option<bool> {
        let raw_assignment = self.assignment[variable.id().as_index()];
        raw_assignment.map(|val| val ^ !variable.positive())
    }

    /// Returns a forced variable in a unit clause.
    fn forced_assignment(&self, clause_index: usize) -> Option<Variable> {
        let clause = &self.formula.clauses()[clause_index];
        let stat = &self.clause_stats[clause_index];
        if stat.satisfied == 0 && stat.unsatisfied == clause.num_variables() - 1 {
            for variable in clause.iter() {
                if self.assigned_value(variable).is_none() {
                    return Some(variable);
                }
            }
            unreachable!()
        } else {
            None
        }
    }

    /// Finds the next unit clause if exists and returns the forced variable.
    fn search_unit_clause(&self) -> Option<Variable> {
        for clause_index in 0..self.formula.clauses().len() {
            if let Some(variable) = self.forced_assignment(clause_index) {
                return Some(variable);
            }
        }

        None
    }

    fn first_unassigned(&self) -> VariableId {
        let index = self
            .assignment
            .iter()
            .position(|assigned| assigned.is_none())
            .unwrap();

        VariableId::from_index(index)
    }

    fn assign_variable(&mut self, variable: Variable) {
        self.assigned_stack.push(variable);
        self.assignment[variable.id().as_index()] = Some(variable.positive());

        for &clause_index in &self.watch[variable] {
            let mut stat = &mut self.clause_stats[clause_index];

            if stat.satisfied == 0 {
                self.satisfied_clauses += 1;
            }
            stat.satisfied += 1;
        }

        for &clause_index in &self.watch[!variable] {
            let clause = &self.formula.clauses()[clause_index];
            let mut stat = &mut self.clause_stats[clause_index];

            stat.unsatisfied += 1;
            if stat.unsatisfied == clause.num_variables() {
                self.unsatisfied_clauses += 1;
            }
        }
    }

    fn pop_assignment(&mut self) {
        let variable = self.assigned_stack.pop().unwrap();
        self.assignment[variable.id().as_index()] = None;

        for &clause_index in &self.watch[variable] {
            let mut stat = &mut self.clause_stats[clause_index];

            if stat.satisfied == 1 {
                self.satisfied_clauses -= 1;
            }
            stat.satisfied -= 1;
        }

        for &clause_index in &self.watch[!variable] {
            let clause = &self.formula.clauses()[clause_index];
            let mut stat = &mut self.clause_stats[clause_index];

            if stat.unsatisfied == clause.num_variables() {
                self.unsatisfied_clauses -= 1;
            }
            stat.unsatisfied -= 1;
        }
    }
}

/// Solve CNF SAT problem with DPLL algorithm.
/// Returns `Some(Model)` if satisfiable, `None` otherwise.
pub fn solve(formula: Cnf) -> Option<Model> {
    let mut solver = DpllSolver::new(formula);

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

        // See if there is a forced assignment.
        if let Some(variable) = solver.search_unit_clause() {
            solver.assign_variable(variable);
            if let Some(assignment) = solve_inner(solver) {
                return Some(assignment);
            }
            solver.pop_assignment();

            None
        } else {
            // Try the first unassigned variable.
            // Note: This is an inefficient heuristics.
            let variable_id = solver.first_unassigned();
            let variable = Variable::new(variable_id, true);

            solver.assign_variable(variable);
            if let Some(assignment) = solve_inner(solver) {
                return Some(assignment);
            }
            solver.pop_assignment();

            solver.assign_variable(!variable);
            if let Some(assignment) = solve_inner(solver) {
                return Some(assignment);
            }
            solver.pop_assignment();

            None
        }
    }

    let assignment = solve_inner(&mut solver);
    assignment.map(|assignment| Model::new(solver.formula, assignment))
}
