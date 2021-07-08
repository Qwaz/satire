use paste::paste;

use crate::{
    parser::parse_file,
    solver::{DpllSolver, Solver},
};

macro_rules! sat_testcase_with_solver {
    ($solver:ident, $dir:ident, $name: ident) => {
        paste! {
            #[test]
            fn [< $solver:lower _ $dir _ $name >]() {
                let formula = parse_file(
                    concat!("testcases/", stringify!($dir), "/", stringify!($name), ".cnf")
                ).unwrap();
                let solver = $solver::new(formula);
                assert!(solver.solve().is_some());
            }
        }
    };
}

macro_rules! unsat_testcase_with_solver {
    ($solver:ident, $dir:ident, $name:ident) => {
        paste! {
            #[test]
            fn [< $solver:lower _ $dir _ $name >]() {
                let formula = parse_file(
                    concat!("testcases/", stringify!($dir), "/", stringify!($name), ".cnf")
                ).unwrap();
                let solver = $solver::new(formula);
                assert!(solver.solve().is_none());
            }
        }
    };
}

macro_rules! sat_testcase {
    ($dir:ident, $name:ident) => {
        sat_testcase_with_solver!(DpllSolver, $dir, $name);
    };
}

macro_rules! unsat_testcase {
    ($dir:ident, $name:ident) => {
        unsat_testcase_with_solver!(DpllSolver, $dir, $name);
    };
}

// satch testcases
sat_testcase!(satch_cnfs, true);
sat_testcase!(satch_cnfs, false);

sat_testcase!(satch_cnfs, unit1);
sat_testcase!(satch_cnfs, unit2);
sat_testcase!(satch_cnfs, unit3);
sat_testcase!(satch_cnfs, unit4);
unsat_testcase!(satch_cnfs, unit5);
unsat_testcase!(satch_cnfs, unit6);
sat_testcase!(satch_cnfs, unit7);
unsat_testcase!(satch_cnfs, unit8);
unsat_testcase!(satch_cnfs, unit9);

unsat_testcase!(satch_cnfs, full2);
unsat_testcase!(satch_cnfs, full3);
unsat_testcase!(satch_cnfs, full4);

unsat_testcase!(satch_cnfs, add4);
unsat_testcase!(satch_cnfs, add8);
unsat_testcase!(satch_cnfs, add16);
unsat_testcase!(satch_cnfs, add32);
unsat_testcase!(satch_cnfs, add64);
unsat_testcase!(satch_cnfs, add128);

unsat_testcase!(satch_cnfs, ph2);
unsat_testcase!(satch_cnfs, ph3);
unsat_testcase!(satch_cnfs, ph4);
unsat_testcase!(satch_cnfs, ph5);
unsat_testcase!(satch_cnfs, ph6);

sat_testcase!(satch_cnfs, prime4);
sat_testcase!(satch_cnfs, prime9);
sat_testcase!(satch_cnfs, prime25);
sat_testcase!(satch_cnfs, prime49);
sat_testcase!(satch_cnfs, prime121);
sat_testcase!(satch_cnfs, prime169);
sat_testcase!(satch_cnfs, prime289);
sat_testcase!(satch_cnfs, prime361);
sat_testcase!(satch_cnfs, prime529);
sat_testcase!(satch_cnfs, prime841);
sat_testcase!(satch_cnfs, prime961);
sat_testcase!(satch_cnfs, prime1369);
sat_testcase!(satch_cnfs, prime1681);
sat_testcase!(satch_cnfs, prime1849);
sat_testcase!(satch_cnfs, prime2209);

unsat_testcase!(satch_cnfs, prime65537);
unsat_testcase!(satch_cnfs, prime4294967297);

sat_testcase!(satch_cnfs, sqrt2809);
sat_testcase!(satch_cnfs, sqrt3481);
sat_testcase!(satch_cnfs, sqrt3721);
sat_testcase!(satch_cnfs, sqrt4489);
sat_testcase!(satch_cnfs, sqrt5041);
sat_testcase!(satch_cnfs, sqrt5329);
sat_testcase!(satch_cnfs, sqrt6241);
sat_testcase!(satch_cnfs, sqrt6889);
sat_testcase!(satch_cnfs, sqrt7921);
sat_testcase!(satch_cnfs, sqrt9409);
sat_testcase!(satch_cnfs, sqrt10201);
sat_testcase!(satch_cnfs, sqrt10609);
sat_testcase!(satch_cnfs, sqrt11449);
sat_testcase!(satch_cnfs, sqrt11881);
sat_testcase!(satch_cnfs, sqrt12769);
sat_testcase!(satch_cnfs, sqrt16129);
sat_testcase!(satch_cnfs, sqrt63001);
sat_testcase!(satch_cnfs, sqrt259081);
sat_testcase!(satch_cnfs, sqrt1042441);
