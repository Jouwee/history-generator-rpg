use std::{cell::{Ref, RefCell, RefMut}, cmp::Ordering};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, PartialOrd)]
// TODO: Remove pub
pub struct Id(pub usize);

impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Id {

    pub fn seq(&self) -> usize {
        return self.0;
    }

    pub fn next(&mut self) -> Id {
        let clone = self.clone();
        self.0 = self.0 + 1;
        clone
    }
}

#[derive(Debug)]
pub struct HistoryVec<T> {
    vector: Vec<RefCell<T>>,
}

impl<T> HistoryVec<T> {
    pub fn new() -> HistoryVec<T> {
        HistoryVec {
            vector: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        return self.vector.len()
    }

    pub fn get(&self, id: &Id) -> Ref<T> {
        self.vector.get(id.0).expect("Invalid ID").borrow()
    }

    pub fn get_mut(&self, id: &Id) -> RefMut<T> {
        self.vector.get(id.0).expect("Invalid ID").borrow_mut()
    }

    pub fn insert(&mut self, value: T) -> Id {
        let id = Id(self.len());
        self.vector.push(RefCell::new(value));
        return id
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id, &RefCell<T>)> {
        self.vector.iter().enumerate().map(|(i, v)| (Id(i), v))
    }

}