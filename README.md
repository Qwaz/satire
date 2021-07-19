# Satire

Satire is an educational (i.e., not feature-complete) SAT solver that may evolve into something else (or not) in the future.

I wrote Satire primarily for learning about SAT solver internals by writing one.
However, I also tried hard to separate different concepts of SAT solvers in different modules,
so it might be helpful for people who found existing SAT solver code difficult to read
or who is learning how to organize code in Rust.

## How to use

Satire accepts limited form of DIMAC CNF files.

```
satire [dpll|cdcl] check testcases/satch_cnfs/add4.cnf
```

To run the entire test suite, use `cargo test`.

```
# Run all tests (note: it doesn't pass all tests in time)
cargo test --release

# Run all tests only with cdcl solver (note: it doesn't pass all tests in time)
cargo test --release -- cdcl
```

## Known limitations

* DPLL solver uses recursion which unnecessarily causes function call overhead.
* CDCL solver uses O(1) data structure for literal marking, but it is often slower than linear search due to the constant overhead.
* Binary heap used for VSIDS scoring scheme is quite inefficient. It could be fixed by custom binary heap implementation.
* No clause minimization.
* No restart.

## References

* [SATCH](https://github.com/arminbiere/satch)
* [MiniSat](https://github.com/niklasso/minisat)
* [splr](https://github.com/shnarazk/splr)
