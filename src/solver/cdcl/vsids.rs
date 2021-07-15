use std::{cmp::Ordering, collections::BTreeSet};

use crate::formula::{Clause, Variable};

use super::tracker::Tracker;

#[derive(Clone, Copy)]
struct VecEntry {
    score: f64,
    nonce: f64,
}

impl VecEntry {
    pub fn new(score: f64) -> Self {
        VecEntry {
            score,
            nonce: rand::random(),
        }
    }

    /// Update the score by delta, change the nonce, and return the updated score.
    pub fn update(&mut self, delta: f64) -> f64 {
        self.score += delta;
        self.nonce = rand::random();
        self.score
    }
}

#[derive(PartialEq, Clone, Copy)]
struct SetEntry {
    variable: Variable,
    score: f64,
    nonce: f64,
}

impl SetEntry {
    pub fn from_vec_entry(variable: Variable, vec_entry: VecEntry) -> Self {
        SetEntry {
            variable,
            score: vec_entry.score,
            nonce: vec_entry.nonce,
        }
    }
}

impl Eq for SetEntry {}

impl PartialOrd for SetEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SetEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self
            .score
            .partial_cmp(&other.score)
            .expect("NaN in heap entry");
        if ordering != Ordering::Equal {
            return ordering;
        }

        let ordering = self
            .nonce
            .partial_cmp(&other.nonce)
            .expect("NaN in heap entry");
        if ordering != Ordering::Equal {
            return ordering;
        }

        Ordering::Equal
    }
}

/// Variable State Independent Decaying Sum (VSIDS) heuristic.
/// Based on MiniSAT implementation.
pub struct VsidsScoring {
    current_rate: f64,
    scores: Vec<VecEntry>,
    btree: BTreeSet<SetEntry>,
}

impl VsidsScoring {
    const DECAY_RATE: f64 = 0.95;
    const REBALANCE_THRESHOLD: f64 = 1e100;

    pub fn new(tracker: &Tracker) -> Self {
        let num_variables = tracker.num_variables();

        let mut scores = Vec::with_capacity(num_variables);
        let mut btree = BTreeSet::new();

        for index in 0..num_variables {
            let variable = Variable::from_index(index).unwrap();
            let vec_entry = VecEntry::new(tracker.variable_occurrence(variable) as f64);
            scores.push(vec_entry);
            btree.insert(SetEntry::from_vec_entry(variable, vec_entry));
        }

        VsidsScoring {
            current_rate: 1.0,
            scores,
            btree,
        }
    }

    fn bump_score(&mut self, variable: Variable) {
        let present = self.btree.remove(&self.set_entry(variable));

        let new_score = self.scores[variable.index()].update(self.current_rate);

        if present {
            self.btree.insert(self.set_entry(variable));
        }

        if new_score >= Self::REBALANCE_THRESHOLD {
            self.rebalance();
        }
    }

    fn rebalance(&mut self) {
        self.current_rate /= Self::REBALANCE_THRESHOLD;
        for index in 0..self.scores.len() {
            let variable = Variable::from_index(index).unwrap();
            let present = self.btree.remove(&self.set_entry(variable));
            self.scores[index].score /= Self::REBALANCE_THRESHOLD;
            if present {
                self.btree.insert(self.set_entry(variable));
            }
        }
    }

    fn set_entry(&self, variable: Variable) -> SetEntry {
        SetEntry::from_vec_entry(variable, self.scores[variable.index()])
    }

    pub fn insert(&mut self, variable: Variable) {
        trace!("VSIDS insert {}", variable);
        self.btree.insert(self.set_entry(variable));
    }

    pub fn remove(&mut self, variable: Variable) {
        trace!("VSIDS remove {}", variable);
        self.btree.remove(&self.set_entry(variable));
    }

    pub fn top(&mut self) -> Variable {
        let variable = self.btree.iter().next().unwrap().variable;
        variable
    }

    pub fn decay(&mut self) {
        self.current_rate /= Self::DECAY_RATE;
    }

    pub fn learn_clause(&mut self, clause: &Clause) {
        for literal in clause.iter() {
            self.bump_score(literal.variable());
        }
    }
}
