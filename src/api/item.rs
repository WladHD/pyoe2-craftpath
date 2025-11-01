use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};

use rustc_hash::{FxBuildHasher, FxHasher};

use crate::{
    api::types::{AffixSpecifier, BaseItemId, ItemLevel, ItemRarityEnum, THashSet},
    derive_DebugDisplay,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(
    feature = "python",
    pyo3(eq, weakref, from_py_object, frozen, hash, get_all, str)
)]
pub struct ItemSnapshot {
    pub item_level: ItemLevel,
    pub rarity: ItemRarityEnum,
    pub base_id: BaseItemId,
    pub affixes: THashSet<AffixSpecifier>,
    pub corrupted: bool,
    pub allowed_sockets: u8,
    pub sockets: THashSet<AffixSpecifier>,
}

impl Hash for ItemSnapshot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.item_level.hash(state);
        self.rarity.hash(state);
        self.base_id.hash(state);
        self.corrupted.hash(state);
        self.allowed_sockets.hash(state);

        fn hash_set_unordered<T: Hash>(set: &HashSet<T, FxBuildHasher>) -> u64 {
            let mut combined: u64 = 0;
            for v in set {
                let mut h = FxHasher::default();
                v.hash(&mut h);
                combined ^= h.finish();
            }
            combined
        }

        let affix_hash = hash_set_unordered(&self.affixes);
        let socket_hash = hash_set_unordered(&self.sockets);

        affix_hash.hash(state);
        socket_hash.hash(state);
    }
}

derive_DebugDisplay!(ItemSnapshot);
