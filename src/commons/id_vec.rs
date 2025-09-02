use std::{cell::{Ref, RefCell, RefMut}, ops::{Deref, DerefMut}};

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

    pub(crate) fn get<K>(&'_ self, id: &K) -> Identified<'_, K, V> where K: Id {
        let value = self.vector.get(id.as_usize())
            .expect("Using IdVec should be safe to unwrap")
            .borrow();
        return Identified::new(*id, value)
    }

    pub(crate) fn get_mut<K>(&'_ self, id: &K) -> IdentifiedMut<'_, K, V> where K: Id {
        let value = self.vector.get(id.as_usize())
            .expect("Using IdVec should be safe to unwrap")
            .borrow_mut();
        return IdentifiedMut::new(*id, value)
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

pub(crate) struct Identified<'a, I, V> {
    _id: I,
    value: Ref<'a, V>
}

impl<'a, I, V> Identified<'a, I, V> {

    pub(crate) fn new(id: I, value: Ref<'a, V>) -> Self {
        Self { _id: id, value }
    }

    // pub(crate) fn id(&self) -> &I {
    //     return &self.id;
    // }

}

impl<'a, I, V> Deref for Identified<'a, I, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        return self.value.deref();
    }
}

impl<'a, I, V> AsRef<V> for Identified<'a, I, V> {
    fn as_ref(&self) -> &V {
        return &self.value;
    }
}


pub(crate) struct IdentifiedMut<'a, I, V> {
    _id: I,
    value: RefMut<'a, V>
}

impl<'a, I, V> IdentifiedMut<'a, I, V> {

    pub(crate) fn new(id: I, value: RefMut<'a, V>) -> Self {
        Self { _id: id, value }
    }

    // pub(crate) fn id(&self) -> &I {
    //     return &self.id;
    // }

}

impl<'a, I, V> Deref for IdentifiedMut<'a, I, V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        return self.value.deref();
    }
}

impl<'a, I, V> DerefMut for IdentifiedMut<'a, I, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return self.value.deref_mut();
    }
}

impl<'a, I, V> AsRef<V> for IdentifiedMut<'a, I, V> {
    fn as_ref(&self) -> &V {
        return &self.value;
    }
}

impl<'a, I, V> AsMut<V> for IdentifiedMut<'a, I, V> {
    fn as_mut(&mut self) -> &mut V {
        return &mut self.value;
    }
}