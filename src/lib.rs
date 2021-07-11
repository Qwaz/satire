#[macro_use]
extern crate log;

pub mod formula;
pub mod parser;
pub mod prelude;
pub mod report;
pub mod solver;

#[cfg(test)]
mod tests;
