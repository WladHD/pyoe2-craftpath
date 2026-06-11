use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

use rustc_hash::{FxBuildHasher, FxHasher};

pub fn hash_set_unordered<T: Hash>(set: &HashSet<T, FxBuildHasher>) -> u64 {
    let mut combined: u64 = 0;
    for v in set {
        let mut h = FxHasher::default();
        v.hash(&mut h);
        combined ^= h.finish();
    }
    combined
}

pub fn hash_value<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}
