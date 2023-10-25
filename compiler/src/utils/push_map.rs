use derive_more::Index;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Index;

#[derive(Index)]
pub struct PushMap<K: Hash + Eq + Clone, V>(HashMap<K, V>);

impl<K: Hash + Eq + Clone, V> Default for PushMap<K, V> {
    fn default() -> Self {
        PushMap(HashMap::default())
    }
}

impl<K: Hash + Eq + Clone, V> From<HashMap<K, V>> for PushMap<K, V> {
    fn from(value: HashMap<K, V>) -> Self {
        PushMap(value)
    }
}

impl<K: Hash + Eq + Clone + Debug, V> PushMap<K, V> {
    pub(crate) fn from_iter(value: impl Iterator<Item = (K, V)>) -> Self {
        PushMap(value.collect())
    }

    pub fn contains(&self, k: &K) -> bool {
        self.0.contains_key(k)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.0.get(k)
    }

    pub fn push<O>(&mut self, k: K, v: V, sub: impl FnOnce(&mut Self) -> O) -> O {
        let old = self.0.insert(k.clone(), v);

        let o = sub(self);

        if let Some(old) = old {
            self.0.insert(k, old);
        } else {
            self.0.remove(&k);
        }

        o
    }

    pub fn remove<O>(&mut self, k: K, scope: impl FnOnce(&mut Self) -> O) -> O {
        let old = self.0.remove(&k);

        let o = scope(self);

        if let Some(old) = old {
            self.0.insert(k, old);
        } else {
            self.0.remove(&k);
        }

        o
    }

    pub fn push_iter<O>(
        &mut self,
        iterator: impl Iterator<Item = (K, V)>,
        sub: impl FnOnce(&mut Self) -> O,
    ) -> O {
        let old = iterator
            .map(|(k, v)| (k.clone(), self.0.insert(k, v)))
            .collect::<Vec<_>>();

        let o = sub(self);

        for (k, old) in old {
            if let Some(old) = old {
                self.0.insert(k, old);
            } else {
                self.0.remove(&k);
            }
        }

        o
    }
}
