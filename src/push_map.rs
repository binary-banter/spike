use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

#[derive(Default)]
pub struct PushMap<K: Hash + Eq + Clone, V>(HashMap<K, V>);

impl<K: Hash + Eq + Clone, V> PushMap<K, V> {
    pub fn push<O>(&mut self, k: K, v: V, scope: impl FnOnce(&mut Self) -> O) -> O {
        let old = self.0.insert(k.clone(), v);

        let v = scope(self);

        if let Some(old) = old {
            self.0.insert(k, old);
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