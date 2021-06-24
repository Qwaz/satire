use satire_dpll::{parser::parse_file, report::Report};

fn main() -> Result<(), Report> {
    let cnf1 = parse_file("testcases/satch-cnfs/ph2.cnf")?;
    println!("{}", cnf1);

    let cnf2 = parse_file("dpll/src/main.rs")?;
    println!("{}", cnf2);

    Ok(())
}
