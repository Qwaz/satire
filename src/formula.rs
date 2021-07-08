/*!
A module to represent conjunctive normal form formula.
*/

use std::{convert::TryInto, fmt::Display, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Snafu)]
pub enum VariableParseError {
    #[snafu(display("Failed to parse Variable ID"))]
    ParseIntError { source: std::num::ParseIntError },
    #[snafu(display(
        "Variable ID {} is out of range (must be within 1 to {})",
        num,
        Variable::MAX_VARIABLE_INDEX + 1
    ))]
    RangeError { num: usize },
}

/// Newtype wrapper for variable ID.
/// Internally uses 0-based index, but uses 1-based index for printing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variable(u32);

impl Variable {
    pub const MAX_VARIABLE_INDEX: usize = std::u32::MAX as usize;
}

impl Variable {
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Creates a variable from an index.
    /// Returns `None` if the index is invalid.
    pub fn from_index(index: usize) -> Option<Self> {
        if index > Variable::MAX_VARIABLE_INDEX {
            return None;
        }
        Some(Variable(index.try_into().unwrap()))
    }
}

impl FromStr for Variable {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.parse::<usize>().context(ParseIntError)?;
        let index = num.checked_sub(1).context(RangeError { num })?;
        Variable::from_index(index).context(RangeError { num: index })
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{}", self.0 as usize + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Literal {
    variable: Variable,
    positive: bool,
}

impl Literal {
    pub fn new(variable: Variable, positive: bool) -> Self {
        Literal { variable, positive }
    }

    pub fn variable(self) -> Variable {
        self.variable
    }

    pub fn index(self) -> usize {
        self.variable.index()
    }

    pub fn positive(self) -> bool {
        self.positive
    }

    pub fn value(self, assignments: &[bool]) -> bool {
        if self.positive {
            assignments[self.index()]
        } else {
            !assignments[self.index()]
        }
    }

    pub fn partial_value(self, assignments: &[Option<bool>]) -> Option<bool> {
        match assignments[self.index()] {
            Some(val) => Some(if self.positive { val } else { !val }),
            None => None,
        }
    }
}

impl FromStr for Literal {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (positive, variable) = if s.starts_with('-') {
            (false, s[1..].parse()?)
        } else {
            (true, s.parse()?)
        };

        Ok(Literal { variable, positive })
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if self.positive { "" } else { "¬" },
            self.variable
        )
    }
}

impl std::ops::Not for Literal {
    type Output = Literal;

    fn not(self) -> Self::Output {
        Literal {
            variable: self.variable,
            positive: !self.positive,
        }
    }
}

/// Disjunction variables
#[derive(Debug, Clone)]
pub struct Clause {
    literals: Vec<Literal>,
}

impl Clause {
    pub fn new(literals: Vec<Literal>) -> Self {
        Self { literals }
    }

    pub fn num_literals(&self) -> usize {
        self.literals.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = Literal> + '_ {
        self.literals.iter().copied()
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        let mut iter = self.literals.iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
        }
        for variable in iter {
            write!(f, " ∨ {}", variable)?;
        }

        write!(f, ")")?;

        Ok(())
    }
}

/// Formula representation in Conjunctive Normal Form
#[derive(Debug, Clone)]
pub struct Cnf {
    num_variables: usize,
    clauses: Vec<Clause>,
}

impl Cnf {
    pub fn new(num_variables: usize) -> Self {
        assert!(num_variables <= Variable::MAX_VARIABLE_INDEX + 1);

        Cnf {
            num_variables,
            clauses: Vec::new(),
        }
    }

    pub fn num_variables(&self) -> usize {
        self.num_variables
    }

    pub fn clauses(&self) -> &Vec<Clause> {
        &self.clauses
    }

    /// Adds a clause to the current formula.
    ///
    /// # Panics
    ///
    /// Panics when `clause` contains invalid literals.
    pub fn add_clause(&mut self, clause: Clause) {
        // sanity check - variables are in-range
        assert!(clause
            .iter()
            .all(|literal| literal.index() < self.num_variables));

        self.clauses.push(clause);
    }
}

impl Display for Cnf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CNF with {} variables (", self.num_variables)?;

        let mut iter = self.clauses.iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
        }
        for variable in iter {
            write!(f, " ∧ {}", variable)?;
        }

        write!(f, ")")?;

        Ok(())
    }
}

/// Represents a satisfying assignment for a formula.
#[derive(Debug)]
pub struct Model {
    formula: Cnf,
    assignment: Vec<bool>,
}

impl Model {
    /// Creates a new model from a formula and an assignment.
    ///
    /// # Panics
    ///
    /// Panics when `assignment` is invalid (e.g., length mismatch, unsatisfying).
    pub fn new(formula: Cnf, assignment: Vec<bool>) -> Self {
        assert!(assignment.len() == formula.num_variables());

        // verify model validity
        for clause in &formula.clauses {
            assert!(clause.iter().any(|literal| literal.value(&assignment)));
        }

        Model {
            formula,
            assignment,
        }
    }

    pub fn formula(&self) -> &Cnf {
        &self.formula
    }

    pub fn assignment(&self) -> &[bool] {
        &self.assignment
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model for {}\nAssignment:", self.formula)?;
        for (idx, &val) in self.assignment.iter().enumerate() {
            write!(f, "\n  {}: {}", Variable::from_index(idx).unwrap(), val)?;
        }

        Ok(())
    }
}
