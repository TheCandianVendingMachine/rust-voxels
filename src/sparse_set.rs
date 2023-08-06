/*
    A roguelike game created for a fun exercise
    Copyright (C) 2023  Bailey Danyluk

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementHandle(pub usize);

impl From<ElementHandle> for usize {
    fn from(value: ElementHandle) -> usize {
        value.0
    }
}

impl From<usize> for ElementHandle {
    fn from(value: usize) -> ElementHandle {
        ElementHandle(value)
    }
}

pub struct SparseSet<T> {
    sparse: Vec<ElementHandle>,
    dense: Vec<ElementHandle>,
    dense_objects: Vec<T>,
    tombstone: ElementHandle
}

impl<T> SparseSet<T> {
    pub fn new(length: usize) -> SparseSet<T> {
        let tombstone = ElementHandle(length);

        let mut sparse = Vec::new();
        sparse.resize(length + 1, tombstone.into());

        SparseSet {
            sparse,
            dense: Vec::new(),
            dense_objects: Vec::new(),
            tombstone
        }
    }

    pub fn push(&mut self, element_id: ElementHandle, element: T) -> &mut T {
        if !self.contains(element_id.into()) {
            let pos = self.dense.len().into();
            self.dense.push(element_id);
            self.dense_objects.push(element);
            self.sparse[element_id.0] = pos;
        }
        self.get_mut(element_id.into()).unwrap()
    }

    pub fn remove(&mut self, element_id: ElementHandle) -> (ElementHandle, Option<T>) {
        if !self.contains(element_id) {
            return (self.tombstone, None)
        }

        let size = self.dense.len() - 1;
        let last = *self.dense.last().unwrap();

        self.dense.swap(size, self.sparse[element_id.0].into());
        self.dense_objects.swap(size, self.sparse[element_id.0].into());

        self.sparse.swap(last.0, element_id.0);
        self.sparse[element_id.0] = self.tombstone;

        (self.dense.pop().unwrap(), Some(self.dense_objects.pop().unwrap()))
    }

    pub fn contains(&self, element: ElementHandle) -> bool {
        element < self.tombstone &&
            self.sparse[element.0].0 < self.dense.len() && 
            self.sparse[element.0] != self.tombstone
    }

    pub fn clear(&mut self) {
        self.dense.clear();
        self.dense_objects.clear();
        self.sparse = self.sparse.iter_mut()
            .map(|_| -> ElementHandle { self.tombstone }).collect();
    }

    pub fn get(&self, element: ElementHandle) -> Option<&T> {
        if !self.contains(element) {
            return None
        }
        Some(&self.dense_objects[self.sparse[element.0].0])
    }

    pub fn get_mut(&mut self, element: ElementHandle) -> Option<&mut T> {
        if !self.contains(element) {
            return None
        }
        Some(&mut self.dense_objects[self.sparse[element.0].0])
    }

    pub fn get_all_elements(&self) -> Vec<ElementHandle> {
        self.sparse.iter().filter(|s| { **s != self.tombstone }).copied().collect()
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SPARSE_SET_TEST_SIZE: usize = 100;

    #[test]
    fn test_push() {
        let mut set = SparseSet::new(SPARSE_SET_TEST_SIZE);
        for i in 0..SPARSE_SET_TEST_SIZE {
            set.push(ElementHandle(i), 2*i);
            assert_eq!(set.dense[i], ElementHandle(i));
            assert_eq!(set.dense_objects[i], 2*i);
        }
    }

    #[test]
    fn test_remove() {
        let mut set = SparseSet::new(SPARSE_SET_TEST_SIZE);
        for i in 0..SPARSE_SET_TEST_SIZE {
            set.push(ElementHandle(i), i);
        }

        for i in (SPARSE_SET_TEST_SIZE/2)..(SPARSE_SET_TEST_SIZE) {
            assert_eq!(set.remove(ElementHandle(i)), (ElementHandle(i), Some(i)));
        }

        assert_eq!(set.dense.len(), SPARSE_SET_TEST_SIZE/2);
        assert_eq!(set.remove(ElementHandle(SPARSE_SET_TEST_SIZE + 1)), (set.tombstone, None));
    }

    #[test]
    fn test_contains() {
        let mut set = SparseSet::new(SPARSE_SET_TEST_SIZE);
        for i in 0..SPARSE_SET_TEST_SIZE/2 {
            set.push(ElementHandle(2 * i), 4 * i);
        }

        assert_eq!(set.contains(ElementHandle(1)), false);
        assert_eq!(set.contains(ElementHandle(98)), true);
        assert_eq!(set.contains(ElementHandle(SPARSE_SET_TEST_SIZE + 1)), false);
    }

    #[test]
    fn test_get() {
        let mut set = SparseSet::new(SPARSE_SET_TEST_SIZE);
        for i in 0..SPARSE_SET_TEST_SIZE {
            set.push(ElementHandle(i), 3 * i);
        }

        for i in 0..SPARSE_SET_TEST_SIZE {
            assert_eq!(*set.get(ElementHandle(i)).unwrap(), 3 * i);
        }

        for i in 0..SPARSE_SET_TEST_SIZE {
            *set.get_mut(ElementHandle(i)).unwrap() *= 2;
        }

        for i in 0..SPARSE_SET_TEST_SIZE {
            assert_eq!(*set.get(ElementHandle(i)).unwrap(), i * 6);
        }
    }
}
