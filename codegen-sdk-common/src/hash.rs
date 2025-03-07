// Taken from https://github.com/salsa-rs/salsa/blob/9d2a9786c45000f5fa396ad2872391e302a2836a/src/hash.rs#L1
use std::hash::{BuildHasher, Hash};

pub type FxHasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;
pub type FxIndexSet<K> = indexmap::IndexSet<K, FxHasher>;
pub type FxIndexMap<K, V> = indexmap::IndexMap<K, V, FxHasher>;
// pub type FxDashMap<K, V> = dashmap::DashMap<K, V, FxHasher>;
// pub type FxLinkedHashSet<K> = hashlink::LinkedHashSet<K, FxHasher>;
pub type FxHashSet<K> = hashbrown::HashSet<K, FxHasher>;
pub type FxHashMap<K, V> = hashbrown::HashMap<K, V, FxHasher>;
pub fn hash<T: Hash>(t: &T) -> u64 {
    FxHasher::default().hash_one(t)
}
