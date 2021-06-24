/*!
A module to represent conjunctive normal form formula
*/

use std::{fmt::Display, str::FromStr};

/// Newtype wrapper for variable ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariableId(u32);

impl VariableId {
    pub const MAX_VARIABLE_ID: usize = std::u32::MAX as usize;
}

impl FromStr for VariableId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s.parse::<u32>()?;
        Ok(VariableId(inner))
    }
}

impl From<u32> for VariableId {
    fn from(num: u32) -> Self {
        VariableId(num)
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
    negated: bool,
}

impl FromStr for Variable {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (negated, id) = if s.starts_with('-') {
            (true, s[1..].parse()?)
        } else {
            (false, s.parse()?)
        };

        Ok(Variable { id, negated })
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", if self.negated { "-" } else { "" }, self.id)
    }
}

/// Disjunction variables
#[derive(Debug)]
pub struct Clause {
    variables: Vec<Variable>,
}

impl Clause {
    pub fn new(variables: Vec<Variable>) -> Self {
        Self { variables }
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
#[derive(Debug)]
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

    pub fn add_clause(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    pub fn clauses(&self) -> &Vec<Clause> {
        &self.clauses
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
