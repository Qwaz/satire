use crate::formula::{Cnf, Model};

mod dpll;

pub use dpll::DpllSolver;

pub trait Solver {
    /// Creates a new solver instance.
    fn new(formula: Cnf) -> Self;

    /// Solves a CNF SAT problem with the solver.
    /// Returns `Some(Model)` if satisfiable, `None` otherwise.
    fn solve(self) -> Option<Model>;
}
