use std::cell::{Ref, RefCell};

pub trait Id: Clone + Copy {

    fn as_usize(&self) -> usize;
    fn new(id: usize) -> Self;

}

pub struct IdVec<V> {
    vector: Vec<RefCell<V>>,
}

impl<V> IdVec<V> {

    pub fn new() -> IdVec<V> {
        return IdVec { vector: vec!() }
    }

    pub fn add<K>(&mut self, value: V) -> K where K: Id {
        let id = K::new(self.vector.len());
        self.vector.push(RefCell::new(value));
        return id
    }

    pub fn get<K>(&self, id: &K) -> Ref<V> where K: Id {
        return self.vector.get(id.as_usize())
            .expect("Using IdVec should be safe to unwrap")
            .borrow()
    }

}
