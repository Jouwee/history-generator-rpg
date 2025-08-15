use std::cell::{Ref, RefCell, RefMut};

use serde::{Deserialize, Serialize};

pub(crate) trait Id: Clone + Copy {

    fn as_usize(&self) -> usize;
    fn new(id: usize) -> Self;

    #[cfg(test)]
    fn mock(id: usize) -> Self {
        return Self::new(id)
    }

}

#[derive(Serialize, Deserialize)]
pub(crate) struct IdVec<V> {
    vector: Vec<RefCell<V>>,
}

impl<V> IdVec<V> {

    pub(crate) fn new() -> IdVec<V> {
        return IdVec { vector: vec!() }
    }

    pub(crate) fn add<K>(&mut self, value: V) -> K where K: Id {
        let id = K::new(self.vector.len());
        self.vector.push(RefCell::new(value));
        return id
    }

    pub(crate) fn get<K>(&'_ self, id: &K) -> Ref<'_, V> where K: Id {
        return self.vector.get(id.as_usize())
            .expect("Using IdVec should be safe to unwrap")
            .borrow()
    }

    pub(crate) fn get_mut<K>(&'_ self, id: &K) -> RefMut<'_, V> where K: Id {
        return self.vector.get(id.as_usize())
            .expect("Using IdVec should be safe to unwrap")
            .borrow_mut()
    }

    pub(crate) fn len(&self) -> usize {
        return self.vector.len()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, RefCell<V>>{
        return self.vector.iter()
    }

    pub(crate) fn iter_id_val<K>(&self) -> impl Iterator<Item = (K, &RefCell<V>)> where K: Id {
        return self.vector.iter().enumerate().map(|(i, val)| (K::new(i), val))
    }

    pub(crate) fn iter_ids<K>(&self) -> impl Iterator<Item = K> where K: Id {
        return (0..self.len()).map(|idx| K::new(idx))
    }

}

impl<V> Default for IdVec<V> {
    fn default() -> Self {
        Self::new()
    }
}