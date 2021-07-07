use std::{env::args, path::Path};

use satire::{
    formula::Model,
    parser::{self, parse_file},
    prelude::*,
    report::Report,
    solver::{DpllSolver, Solver},
};

fn usage_string() -> String {
    format!(
        "Usage: {} <solver_name> <command>

solver_name: dpll, cdcl

command:
    test - test the solver on testcases
    check <file_name> - test the solver with given file",
        args().next().unwrap()
    )
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unknown solver '{}'", name))]
    UnknownSolver { name: String },
    #[snafu(display("Unknown command '{}'", name))]
    UnknownCommand { name: String },
    #[snafu(display("Failed to parse CNF"))]
    ParserError { source: parser::Error },
    #[snafu(display("Incorrect usage\n\n{}", usage_string()))]
    UsageError,
}

fn solve_path<T: Solver>(path: &Path) -> Result<Option<Model>, Error> {
    let formula = parse_file(path).context(ParserError)?;
    let solver = T::new(formula);
    Ok(solver.solve())
}

fn dispatch_command<T: Solver>(args: Vec<String>) -> Result<(), Error> {
    match args.get(0).map(|s| s.as_str()) {
        Some("check") => {
            let path = args.get(1).context(UsageError)?;
            let result = solve_path::<T>(path.as_ref())?;
            if let Some(model) = result {
                println!("SAT {}", model);
            } else {
                println!("UNSAT");
            }
        }
        Some("test") => todo!(),
        Some(name) => UnknownCommand {
            name: name.to_owned(),
        }
        .fail()?,
        None => UsageError.fail()?,
    }

    Ok(())
}

fn main() -> Result<(), Report> {
    let mut args = args();

    // drop arg[0]
    args.next();

    // solver name
    let solver_name = args.next();
    let remaining: Vec<_> = args.collect();

    match solver_name.as_deref() {
        Some("dpll") => dispatch_command::<DpllSolver>(remaining)?,
        Some(name) => UnknownSolver {
            name: name.to_owned(),
        }
        .fail()?,
        None => {
            println!("{}", usage_string());
        }
    }

    Ok(())
}
