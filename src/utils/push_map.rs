use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

pub struct PushMap<K: Hash + Eq + Clone, V>(HashMap<K, V>);

impl<K: Hash + Eq + Clone, V> Default for PushMap<K, V> {
    fn default() -> Self {
        PushMap(HashMap::default())
    }
}

impl<K: Hash + Eq + Clone, V> PushMap<K, V> {
    pub fn contains(&self, k: &K) -> bool {
        self.0.contains_key(k)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.get(k)
    }

    pub fn push<O>(&mut self, k: K, v: V, scope: impl FnOnce(&mut Self) -> O) -> O {
        let old = self.0.insert(k.clone(), v);

        let v = scope(self);

        if let Some(old) = old {
            self.0.insert(k, old);
        } else {
            self.0.remove(&k);
        }

        v
    }
}

impl<K: Hash + Eq + Clone, V> Index<&K> for PushMap<K, V> {
    type Output = V;

    fn index(&self, index: &K) -> &Self::Output {
        &self.0[index]
    }
}
