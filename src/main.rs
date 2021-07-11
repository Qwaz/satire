use std::{env::args, path::Path};

use pretty_env_logger::formatted_builder;
use satire::{
    formula::Model,
    parser::{self, parse_file},
    prelude::*,
    report::Report,
    solver::{CdclSolver, DpllSolver, Solver},
};

fn usage_string() -> String {
    format!(
        "Usage: {} <solver_name> <command>

solver_name: dpll, cdcl

command:
    check <file_name> - test the solver with given file",
        args().next().unwrap()
    )
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unknown solver '{}'\n\n{}", name, usage_string()))]
    UnknownSolver { name: String },
    #[snafu(display("Unknown command '{}'\n\n{}", name, usage_string()))]
    UnknownCommand { name: String },
    #[snafu(display("Failed to parse CNF"))]
    ParserError { source: parser::Error },
    #[snafu(display("Required argument does not exist\n\n{}", usage_string()))]
    MissingArgument,
}

fn solve_path<T: Solver>(path: &Path) -> Result<Option<Model>, Error> {
    let formula = parse_file(path).context(ParserError)?;
    let solver = T::new(formula);
    Ok(solver.solve())
}

fn dispatch_command<T: Solver>(args: Vec<String>) -> Result<(), Error> {
    match args.get(0).map(|s| s.as_str()) {
        Some("check") => {
            let path = args.get(1).context(MissingArgument)?;
            let result = solve_path::<T>(path.as_ref())?;
            if let Some(model) = result {
                println!("SAT {}", model);
            } else {
                println!("UNSAT");
            }
        }
        Some(name) => UnknownCommand {
            name: name.to_owned(),
        }
        .fail()?,
        None => MissingArgument.fail()?,
    }

    Ok(())
}

fn init_logger() {
    let mut builder = formatted_builder();

    if let Ok(s) = ::std::env::var("RUST_LOG") {
        builder.parse_filters(&s);
    } else {
        if cfg!(debug_assertions) {
            builder.parse_filters("satire=debug");
        } else {
            builder.parse_filters("satire=warn");
        }
    }

    builder.try_init().expect("Failed to initialize the logger");
}

fn main() -> Result<(), Report> {
    init_logger();

    let mut args = args();

    // drop arg[0]
    args.next();

    // solver name
    let solver_name = args.next();
    let remaining: Vec<_> = args.collect();

    match solver_name.as_deref() {
        Some("dpll") => dispatch_command::<DpllSolver>(remaining)?,
        Some("cdcl") => dispatch_command::<CdclSolver>(remaining)?,
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
