use std::{
    cell::Cell,
    collections::BTreeSet,
    ops::{Index, IndexMut},
};

use typed_index_collections::TiVec;

use crate::formula::{Clause, Cnf, Literal, Variable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClauseIdx(usize);

impl From<usize> for ClauseIdx {
    fn from(index: usize) -> Self {
        ClauseIdx(index)
    }
}

impl From<ClauseIdx> for usize {
    fn from(index: ClauseIdx) -> Self {
        index.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ClauseCol(usize);

impl From<usize> for ClauseCol {
    fn from(index: usize) -> Self {
        ClauseCol(index)
    }
}

impl From<ClauseCol> for usize {
    fn from(index: ClauseCol) -> Self {
        index.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct VariableCol(usize);

impl From<usize> for VariableCol {
    fn from(index: usize) -> Self {
        VariableCol(index)
    }
}

impl From<VariableCol> for usize {
    fn from(index: VariableCol) -> Self {
        index.0
    }
}

use clause_stat::*;
mod clause_stat {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ClauseStatus {
        Falsified,
        Satisfied,
        Unit,
        Unresolved,
    }

    impl ClauseStatus {
        pub fn from_count(total: usize, satisfied: usize, unsatisfied: usize) -> Self {
            if unsatisfied == total {
                ClauseStatus::Falsified
            } else if satisfied > 0 {
                ClauseStatus::Satisfied
            } else if unsatisfied + 1 == total {
                ClauseStatus::Unit
            } else {
                ClauseStatus::Unresolved
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct ClauseStat {
        /// Number of literals in the clause.
        total: usize,
        /// Satisfied literal count in the clause.
        satisfied: usize,
        /// Unsatisfied literal count in the clause.
        unsatisfied: usize,
        /// Current clause stat
        status: ClauseStatus,
    }

    #[derive(Clone, Copy)]
    pub struct ClauseStatusChange {
        pub old: ClauseStatus,
        pub new: ClauseStatus,
    }

    impl ClauseStat {
        pub fn new(total: usize, satisfied: usize, unsatisfied: usize) -> Self {
            assert!(satisfied.checked_add(unsatisfied).unwrap() <= total);

            ClauseStat {
                total,
                satisfied,
                unsatisfied,
                status: ClauseStatus::from_count(total, satisfied, unsatisfied),
            }
        }

        pub fn status(&self) -> ClauseStatus {
            self.status
        }

        /// Increments the satisfied counter and returns the old status.
        pub fn increment_satisfied(&mut self) -> ClauseStatusChange {
            let old = self.status;
            self.satisfied += 1;
            self.status = ClauseStatus::from_count(self.total, self.satisfied, self.unsatisfied);
            ClauseStatusChange {
                old,
                new: self.status,
            }
        }

        /// Increments the unsatisfied counter and returns the old status.
        pub fn increment_unsatisfied(&mut self) -> ClauseStatusChange {
            let old = self.status;
            self.unsatisfied += 1;
            self.status = ClauseStatus::from_count(self.total, self.satisfied, self.unsatisfied);
            ClauseStatusChange {
                old,
                new: self.status,
            }
        }

        /// Decrements the satisfied counter and returns the old status.
        pub fn decrement_satisfied(&mut self) -> ClauseStatusChange {
            let old = self.status;
            self.satisfied -= 1;
            self.status = ClauseStatus::from_count(self.total, self.satisfied, self.unsatisfied);
            ClauseStatusChange {
                old,
                new: self.status,
            }
        }

        /// Decrements the unsatisfied counter and returns the old status.
        pub fn decrement_unsatisfied(&mut self) -> ClauseStatusChange {
            let old = self.status;
            self.unsatisfied -= 1;
            self.status = ClauseStatus::from_count(self.total, self.satisfied, self.unsatisfied);
            ClauseStatusChange {
                old,
                new: self.status,
            }
        }
    }
}

pub type ClauseSet = BTreeSet<ClauseIdx>;

#[derive(Default)]
struct ClauseStateCache {
    falsified: ClauseSet,
    satisfied: ClauseSet,
    unit: ClauseSet,
    unresolved: ClauseSet,
}

impl Index<ClauseStatus> for ClauseStateCache {
    type Output = ClauseSet;

    fn index(&self, index: ClauseStatus) -> &Self::Output {
        match index {
            ClauseStatus::Falsified => &self.falsified,
            ClauseStatus::Satisfied => &self.satisfied,
            ClauseStatus::Unit => &self.unit,
            ClauseStatus::Unresolved => &self.unresolved,
        }
    }
}

impl IndexMut<ClauseStatus> for ClauseStateCache {
    fn index_mut(&mut self, index: ClauseStatus) -> &mut Self::Output {
        match index {
            ClauseStatus::Falsified => &mut self.falsified,
            ClauseStatus::Satisfied => &mut self.satisfied,
            ClauseStatus::Unit => &mut self.unit,
            ClauseStatus::Unresolved => &mut self.unresolved,
        }
    }
}

impl ClauseStateCache {
    fn new() -> Self {
        Default::default()
    }

    fn handle_change(&mut self, change: ClauseStatusChange, idx: ClauseIdx) {
        if change.old != change.new {
            assert!(self[change.old].remove(&idx));
            assert!(self[change.new].insert(idx));
        }
    }
}

pub struct WatchElement {
    clause_idx: ClauseIdx,
    clause_col: Cell<Option<ClauseCol>>,
}

impl WatchElement {
    fn new(clause_idx: ClauseIdx, clause_col: Option<ClauseCol>) -> Self {
        Self {
            clause_idx,
            clause_col: Cell::new(clause_col),
        }
    }
}

type WatchRow = TiVec<VariableCol, WatchElement>;

struct Watch {
    /// Maps +x_i to clause positions.
    positive: Vec<WatchRow>,
    /// Maps -x_i to clause positions.
    negative: Vec<WatchRow>,
}

impl Watch {
    pub fn new(num_variables: usize) -> Self {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        for _ in 0..num_variables {
            positive.push(TiVec::new());
            negative.push(TiVec::new());
        }

        Watch { positive, negative }
    }
}

impl Index<Literal> for Watch {
    type Output = WatchRow;

    fn index(&self, literal: Literal) -> &Self::Output {
        if literal.positive() {
            &self.positive[literal.index()]
        } else {
            &self.negative[literal.index()]
        }
    }
}

impl IndexMut<Literal> for Watch {
    fn index_mut(&mut self, literal: Literal) -> &mut Self::Output {
        if literal.positive() {
            &mut self.positive[literal.index()]
        } else {
            &mut self.negative[literal.index()]
        }
    }
}

struct TrackedClause {
    stat: ClauseStat,
    original: Clause,
    literals: TiVec<ClauseCol, WatchedLiteral>,
}

struct WatchedLiteral {
    literal: Literal,
    variable_col: VariableCol,
}

pub struct Tracker {
    /// Number of variables.
    num_variables: usize,
    /// The current assignments to variables.
    assignments: Vec<Option<bool>>,
    /// The number of assigned variables.
    assigned_count: usize,
    /// Variable watches.
    watch: Watch,
    /// Inverse-map of watches.
    clauses: TiVec<ClauseIdx, TrackedClause>,
    /// Faster lookup table for clauses.
    clause_cache: ClauseStateCache,
}

impl Tracker {
    pub fn new(num_variables: usize) -> Self {
        Tracker {
            num_variables,
            assignments: vec![None; num_variables],
            assigned_count: 0,
            watch: Watch::new(num_variables),
            clauses: TiVec::new(),
            clause_cache: ClauseStateCache::new(),
        }
    }

    pub fn from_cnf(formula: &Cnf) -> Self {
        let mut tracker = Tracker::new(formula.num_variables());
        for clause in formula.clauses() {
            tracker.add_clause(clause.clone());
        }
        tracker
    }

    pub fn add_clause(&mut self, clause: Clause) {
        let mut satisfied = 0;
        let mut unsatisfied = 0;

        let mut literals = TiVec::new();
        let clause_index = self.clauses.next_key();

        for literal in clause.iter() {
            match literal.partial_value(&self.assignments) {
                Some(true) => {
                    satisfied += 1;
                    self.watch[literal].push(WatchElement::new(clause_index, None));
                }
                Some(false) => {
                    unsatisfied += 1;
                    self.watch[literal].push(WatchElement::new(clause_index, None));
                }
                _ => {
                    let new_clause_col = literals.next_key();
                    let variable_col = self.watch[literal]
                        .push_and_get_key(WatchElement::new(clause_index, Some(new_clause_col)));
                    literals.push(WatchedLiteral {
                        literal,
                        variable_col,
                    });
                }
            }
        }

        let stat = ClauseStat::new(clause.len(), satisfied, unsatisfied);
        self.clause_cache[stat.status()].insert(clause_index);

        self.clauses.push(TrackedClause {
            stat,
            original: clause,
            literals,
        });
    }

    /// Get a reference to the tracker's assignments.
    pub fn assignments(&self) -> &[Option<bool>] {
        self.assignments.as_slice()
    }

    /// Get a reference to the falsified clause set.
    pub fn falsified_clauses(&self) -> &ClauseSet {
        &self.clause_cache.falsified
    }

    /// Get a reference to the satisfied clause set.
    pub fn satisfied_clauses(&self) -> &ClauseSet {
        &self.clause_cache.satisfied
    }

    /// Get a reference to the unit clause set.
    pub fn unit_clauses(&self) -> &ClauseSet {
        &self.clause_cache.unit
    }

    pub fn num_variables(&self) -> usize {
        self.num_variables
    }

    pub fn num_clauses(&self) -> usize {
        self.clauses.len()
    }

    pub fn variable_occurrence(&self, variable: Variable) -> usize {
        self.literal_occurrence(Literal::new(variable, true))
            + self.literal_occurrence(Literal::new(variable, false))
    }

    pub fn literal_occurrence(&self, literal: Literal) -> usize {
        self.watch[literal].len()
    }

    fn fixup_clause(&self, idx: ClauseIdx, col: ClauseCol) {
        if let Some(literal) = self.clauses[idx].literals.get(col) {
            self.watch[literal.literal][literal.variable_col]
                .clause_col
                .set(Some(col));
        }
    }

    /// Set the given literal.
    /// Panic if the literal is already set.
    pub fn set_literal(&mut self, literal: Literal) {
        let old_value = self.assignments[literal.index()].replace(literal.positive());
        assert!(old_value.is_none());
        self.assigned_count += 1;

        for watch in self.watch[literal].iter() {
            // Sets the literal to true
            let clause = &mut self.clauses[watch.clause_idx];

            let change = clause.stat.increment_satisfied();
            self.clause_cache.handle_change(change, watch.clause_idx);

            // Removes the literal from the clause
            let clause_col = watch.clause_col.take().unwrap();
            clause.literals.swap_remove(clause_col);
            self.fixup_clause(watch.clause_idx, clause_col);
        }

        for watch in self.watch[!literal].iter() {
            // Sets the literal to false
            let clause = &mut self.clauses[watch.clause_idx];

            let change = clause.stat.increment_unsatisfied();
            self.clause_cache.handle_change(change, watch.clause_idx);

            // Removes the literal from the clause
            let clause_col = watch.clause_col.take().unwrap();
            clause.literals.swap_remove(clause_col);
            self.fixup_clause(watch.clause_idx, clause_col);
        }
    }

    /// Unset the given variable.
    /// Panic if the literal is not set.
    pub fn unset(&mut self, variable: Variable) {
        let old_value = self.assignments[variable.index()].take().unwrap();
        let literal = Literal::new(variable, old_value);
        self.assigned_count -= 1;

        for (variable_col, watch) in self.watch[literal].iter().enumerate() {
            // Undo literal removal
            let clause = &mut self.clauses[watch.clause_idx];

            let change = clause.stat.decrement_satisfied();
            self.clause_cache.handle_change(change, watch.clause_idx);

            // Adds the literal back to the clause
            let clause_col = clause.literals.push_and_get_key(WatchedLiteral {
                literal,
                variable_col: variable_col.into(),
            });
            watch.clause_col.set(Some(clause_col));
        }

        for (variable_col, watch) in self.watch[!literal].iter().enumerate() {
            // Undo literal removal
            let clause = &mut self.clauses[watch.clause_idx];

            let change = clause.stat.decrement_unsatisfied();
            self.clause_cache.handle_change(change, watch.clause_idx);

            // Adds the literal back to the clause
            let clause_col = clause.literals.push_and_get_key(WatchedLiteral {
                literal: !literal,
                variable_col: variable_col.into(),
            });
            watch.clause_col.set(Some(clause_col));
        }
    }

    /// Get the original clause from the clause index.
    pub fn original_clause(&self, index: ClauseIdx) -> &Clause {
        &self.clauses[index].original
    }

    /// Return the remaining literal in the unit clause.
    /// Panics if the given clause is not a unit clause.
    pub fn get_unit_clause_literal(&self, index: ClauseIdx) -> Literal {
        let tracked_clause = &self.clauses[index];
        assert!(tracked_clause.literals.len() == 1);
        tracked_clause.literals.first().unwrap().literal
    }
}
