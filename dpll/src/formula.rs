/*!
A module to represent conjunctive normal form formula.
*/

use std::{fmt::Display, num::NonZeroU32, str::FromStr};

use crate::prelude::*;

#[derive(Debug, Snafu)]
pub enum VariableError {
    #[snafu(display("Failed to parse Variable ID"))]
    ParseError { source: std::num::ParseIntError },
    #[snafu(display("Variable ID must be non-zero"))]
    ZeroError,
}

/// Newtype wrapper for variable ID.
/// Invariant: 0 < ID <= MAX_VARIABLE_ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariableId(NonZeroU32);

impl VariableId {
    pub const MAX_VARIABLE_ID: usize = std::u32::MAX as usize;
}

impl VariableId {
    pub fn as_index(&self) -> usize {
        (self.0.get() - 1) as usize
    }

    pub fn from_index(index: usize) -> Self {
        let id = index.checked_add(1).unwrap();
        assert!(id <= VariableId::MAX_VARIABLE_ID);
        VariableId(NonZeroU32::new(id as u32).unwrap())
    }
}

impl FromStr for VariableId {
    type Err = VariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.parse::<u32>().context(ParseError)?;
        Ok(VariableId(NonZeroU32::new(num).context(ZeroError)?))
    }
}

impl Display for VariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variable {
    id: VariableId,
    positive: bool,
}

impl Variable {
    pub fn new(id: VariableId, positive: bool) -> Self {
        Variable { id, positive }
    }

    pub fn id(&self) -> VariableId {
        self.id
    }

    pub fn positive(&self) -> bool {
        self.positive
    }
}

impl FromStr for Variable {
    type Err = VariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (negated, id) = if s.starts_with('-') {
            (false, s[1..].parse()?)
        } else {
            (true, s.parse()?)
        };

        Ok(Variable {
            id,
            positive: negated,
        })
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", if self.positive { "" } else { "¬" }, self.id)
    }
}

impl std::ops::Not for Variable {
    type Output = Variable;

    fn not(self) -> Self::Output {
        Variable {
            id: self.id,
            positive: !self.positive,
        }
    }
}

/// Disjunction variables
#[derive(Debug, Clone)]
pub struct Clause {
    variables: Vec<Variable>,
}

impl Clause {
    pub fn new(variables: Vec<Variable>) -> Self {
        Self { variables }
    }

    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = Variable> + '_ {
        self.variables.iter().copied()
    }
}

impl Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        let mut iter = self.variables.iter();
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
        assert!(num_variables <= VariableId::MAX_VARIABLE_ID);

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
            write!(f, "\n  {}: {}", VariableId::from_index(idx), val)?;
        }

        Ok(())
    }
}
