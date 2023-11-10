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

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T> Index<UnionIndex> for UnionFind<T> {
    type Output = T;

    fn index(&self, index: UnionIndex) -> &Self::Output {
        &self.data[index.0]
    }
}

impl<T> IndexMut<UnionIndex> for UnionFind<T> {
    fn index_mut(&mut self, index: UnionIndex) -> &mut Self::Output {
        &mut self.data[index.0]
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
        let ids = uf.add_iter(0..420690);

        for (left, _, right) in ids.tuple_windows() {
            uf.union(left, right);
        }

        assert_ne!(uf.find(UnionIndex(0)), uf.find(UnionIndex(1)));

        for i in (0..42069).step_by(2) {
            assert_eq!(uf.find(UnionIndex(0)), uf.find(UnionIndex(i)))
        }

        for i in (0..42069).skip(1).step_by(2) {
            assert_eq!(uf.find(UnionIndex(1)), uf.find(UnionIndex(i)))
        }

        assert_ne!(uf.find(UnionIndex(0)), uf.find(UnionIndex(1)));
    }
}
