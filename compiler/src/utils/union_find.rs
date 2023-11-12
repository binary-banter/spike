use std::ops::{Index, IndexMut};

pub struct UnionFind<T> {
    parents: Vec<usize>,
    data: Vec<T>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct UnionIndex(usize);

impl<T> UnionFind<T> {
    pub fn new() -> Self {
        Self {
            parents: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn add(&mut self, value: T) -> UnionIndex {
        let index = self.data.len();
        self.parents.push(index);
        self.data.push(value);
        UnionIndex(index)
    }

    pub fn add_iter(
        &mut self,
        values: impl Iterator<Item = T>,
    ) -> impl Iterator<Item = UnionIndex> {
        let i = self.parents.len();
        self.data.extend(values);
        self.parents.extend(i..self.data.len());
        (i..self.data.len()).into_iter().map(UnionIndex)
    }

    pub fn find(&mut self, index: UnionIndex) -> UnionIndex {
        let mut child = index.0;
        let mut parent = self.parents[child];

        // early exit if root
        if parent == child {
            return UnionIndex(parent);
        }

        let parent_parent = self.parents[parent];

        // early exit if one away from root
        if parent_parent == parent {
            return UnionIndex(parent_parent);
        }

        let mut child_indexes = vec![child, parent];
        child = parent_parent;

        // loop until root is found
        loop {
            parent = self.parents[child];
            if parent == child {
                break;
            }
            child_indexes.push(child);
            child = parent;
        }

        // set parent of each child to root
        for child_index in child_indexes {
            self.parents[child_index] = child
        }

        UnionIndex(parent)
    }

    pub fn union(&mut self, a: UnionIndex, b: UnionIndex) -> UnionIndex {
        let a_root = self.find(a);
        let b_root = self.find(b);
        self.parents[b_root.0] = a_root.0;
        self.parents[b.0] = a_root.0;
        a_root
    }

    pub fn try_union_by<E>(
        &mut self,
        a: UnionIndex,
        b: UnionIndex,
        f: impl FnOnce(&T, &T) -> Result<T, E>,
    ) -> Result<UnionIndex, E> {
        let root = self.union(a, b);
        self.data[root.0] = f(&self.data[a.0], &self.data[b.0])?;
        Ok(root)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get(&mut self, index: UnionIndex) -> &T {
        let index = self.find(index).0;
        &self.data[index]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_simple() {
        let mut uf = UnionFind::new();
        let x = uf.add(0);
        let y = uf.add(1);

        assert_eq!(uf.find(x), x);
        assert_eq!(uf.find(y), y);

        uf.union(x, y);

        assert_eq!(uf.find(x), uf.find(y));
    }

    #[test]
    fn test_iter() {
        let mut uf = UnionFind::new();
        let ids = uf.add_iter(0..20);

        for (left, _, right) in ids.tuple_windows() {
            uf.union(left, right);
        }

        assert_ne!(uf.find(UnionIndex(0)), uf.find(UnionIndex(1)));

        for i in (0..20).step_by(2) {
            assert_eq!(uf.find(UnionIndex(0)), uf.find(UnionIndex(i)))
        }

        for i in (0..20).skip(1).step_by(2) {
            assert_eq!(uf.find(UnionIndex(1)), uf.find(UnionIndex(i)))
        }

        assert_ne!(uf.find(UnionIndex(0)), uf.find(UnionIndex(1)));
    }
}
