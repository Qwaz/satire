/*!
A module to represent conjunctive normal form formula.
*/

use std::{convert::TryInto, fmt::Display, num::NonZeroU32, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Snafu)]
pub enum VariableParseError {
    #[snafu(display("Failed to parse Variable ID"))]
    ParseIntError { source: std::num::ParseIntError },
    #[snafu(display(
        "Variable ID {} is out of range (must be within 1 to {})",
        num,
        Variable::MAX_VARIABLE_ID
    ))]
    RangeError { num: usize },
}

/// Newtype wrapper for variable ID.
/// Invariant: 0 < ID <= MAX_VARIABLE_ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variable(NonZeroU32);

impl Variable {
    pub const MAX_VARIABLE_ID: usize = std::u32::MAX as usize;
}

impl Variable {
    pub fn as_index(&self) -> usize {
        (self.0.get() - 1) as usize
    }

    /// Creates a variable from a raw index.
    /// Returns `None` if the index is invalid.
    pub fn from_index(index: usize) -> Option<Self> {
        let id = index.checked_add(1)?;
        if id > Variable::MAX_VARIABLE_ID {
            return None;
        }
        Some(Variable(NonZeroU32::new(id.try_into().ok()?)?))
    }
}

impl FromStr for Variable {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.parse::<usize>().context(ParseIntError)?;
        Variable::from_index(num).context(RangeError { num })
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Literal {
    id: Variable,
    positive: bool,
}

impl Literal {
    pub fn new(id: Variable, positive: bool) -> Self {
        Literal { id, positive }
    }

    pub fn variable(&self) -> Variable {
        self.id
    }

    pub fn positive(&self) -> bool {
        self.positive
    }
}

impl FromStr for Literal {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (negated, id) = if s.starts_with('-') {
            (false, s[1..].parse()?)
        } else {
            (true, s.parse()?)
        };

        Ok(Literal {
            id,
            positive: negated,
        })
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", if self.positive { "" } else { "¬" }, self.id)
    }
}

impl std::ops::Not for Literal {
    type Output = Literal;

    fn not(self) -> Self::Output {
        Literal {
            id: self.id,
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
        assert!(num_variables <= Variable::MAX_VARIABLE_ID);

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

    pub fn add_clause(&mut self, clause: Clause) {
        // TODO: sanity check - variables are in-range
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

        // TODO: verify model validity

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
