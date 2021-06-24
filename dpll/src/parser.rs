use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::Result;

use crate::cnf::{Clause, Cnf};

/// Parse a line to a clause
fn parse_line(line: &str) -> Result<Clause> {
    let mut variables = Vec::new();

    let splitted = line.split(" ").collect::<Vec<_>>();
    if splitted.is_empty() || splitted[splitted.len() - 1] != "0" {
        return Err(anyhow!("Malformed clause line '{}'", line));
    }

    for s in &splitted[..splitted.len() - 1] {
        variables.push(s.parse()?);
    }

    Ok(Clause::new(variables))
}

/// Parses CNF formula from a file
pub fn parse_file(path: impl AsRef<Path>) -> Result<Cnf> {
    let path = path.as_ref();
    let file = BufReader::new(File::open(path)?);

    // skip until we find the problem definition
    let mut lines = file
        .lines()
        .map(|line| line.unwrap())
        .skip_while(|line| !line.starts_with('p'));

    let prob_line = lines
        .next()
        .ok_or_else(|| anyhow!("Problem definition not found in '{}'", path.display()))?;

    let splitted = prob_line.trim().split(" ").collect::<Vec<_>>();

    let malformed_definition = Err(anyhow!(
        "Problem definition malformed - expected 'p cnf <num_variables> <num_clauses>'"
    ));

    // We only support CNF DIMACS format
    if splitted.len() != 4 || splitted[0] != "p" || splitted[1] != "cnf" {
        return malformed_definition;
    }

    let (num_variables, num_clauses) =
        match (splitted[2].parse::<usize>(), splitted[3].parse::<usize>()) {
            (Ok(num_variables), Ok(num_clauses)) => (num_variables, num_clauses),
            _ => return malformed_definition,
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

    if cnf.clauses().len() != num_clauses {
        return Err(anyhow!("The number of actual clauses ({}) does not match the clauses number in the problem line ({})", cnf.clauses().len(), num_clauses));
    }

    Ok(cnf)
}
