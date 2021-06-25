use satire_dpll::{parser::parse_file, report::Report, solver::solve};

fn main() -> Result<(), Report> {
    let formula = parse_file("testcases/satch-cnfs/ph6.cnf")?;
    println!("{}", formula);

    if let Some(model) = solve(formula) {
        println!("SAT {}", model);
    } else {
        println!("UNSAT");
    }

    Ok(())
}
