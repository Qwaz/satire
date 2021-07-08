use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::formula::{Clause, Cnf, Literal, VariableParseError};
use crate::prelude::*;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("I/O error occurred while parsing CNF file '{}'", path.display()))]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Failed to parse line '{}' as clause", clause))]
    MalformedClause { clause: String },
    #[snafu(display("Invalid variable found in clause '{}'", clause))]
    MalformedVariable {
        clause: String,
        source: VariableParseError,
    },
    #[snafu(display("Problem line 'p cnf <num_variables> <num_clauses>' is not found"))]
    MalformedProblemDefinition,
    #[snafu(display(
        "The number of clauses ({}) does not match the clauses number in the problem definition ({})",
        found,
        expected,
    ))]
    ClauseCountMismatch { expected: usize, found: usize },
}

/// Parse a line to a clause
fn parse_line(line: &str) -> Result<Clause, Error> {
    let mut variables = Vec::new();

    let splitted = line.split(" ").collect::<Vec<_>>();

    ensure!(
        !splitted.is_empty() && splitted[splitted.len() - 1] == "0",
        MalformedClause {
            clause: line.to_owned(),
        }
    );

    for s in &splitted[..splitted.len() - 1] {
        variables.push(s.parse::<Literal>().with_context(|| MalformedVariable {
            clause: line.to_owned(),
        })?);
    }

    Ok(Clause::new(variables))
}

/// Parses CNF formula from a file
pub fn parse_file(path: impl AsRef<Path>) -> Result<Cnf, Error> {
    let path = path.as_ref();
    let file = BufReader::new(File::open(path).context(IoError {
        path: path.to_owned(),
    })?);

    // skip until we find the problem definition
    let mut lines = file
        .lines()
        .map(|line| line.unwrap())
        .skip_while(|line| !line.starts_with('p'));

    let prob_line = lines
        .next()
        .ok_or_else(|| MalformedProblemDefinition.build())?;

    let splitted = prob_line.trim().split(" ").collect::<Vec<_>>();

    // We only support CNF DIMACS format
    ensure!(
        splitted.len() == 4 || splitted[0] == "p" || splitted[1] == "cnf",
        MalformedProblemDefinition
    );

    let (num_variables, num_clauses) =
        match (splitted[2].parse::<usize>(), splitted[3].parse::<usize>()) {
            (Ok(num_variables), Ok(num_clauses)) => (num_variables, num_clauses),
            _ => return MalformedProblemDefinition.fail(),
        };

    let mut cnf = Cnf::new(num_variables);

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('c') {
            // empty line, comment
            continue;
        }
        cnf.add_clause(parse_line(&trimmed)?);
    }

    ensure!(
        cnf.clauses().len() + cnf.empty_clause_count() == num_clauses,
        ClauseCountMismatch {
            found: cnf.clauses().len(),
            expected: num_clauses,
        }
    );

    Ok(cnf)
}
