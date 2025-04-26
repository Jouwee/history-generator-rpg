use std::{cell::{Ref, RefCell, RefMut}, cmp::Ordering};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, PartialOrd)]
// TODO: Remove pub
pub(crate) struct Id(pub(crate) usize);

impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Id {

    pub(crate) fn seq(&self) -> usize {
        return self.0;
    }

    pub(crate) fn next(&mut self) -> Id {
        let clone = self.clone();
        self.0 = self.0 + 1;
        clone
    }
}

#[derive(Debug)]
pub(crate) struct HistoryVec<T> {
    vector: Vec<RefCell<T>>,
}

impl<T> HistoryVec<T> {
    pub(crate) fn new() -> HistoryVec<T> {
        HistoryVec {
            vector: Vec::new()
        }
    }

    pub(crate) fn len(&self) -> usize {
        return self.vector.len()
    }

    pub(crate) fn get(&self, id: &Id) -> Ref<T> {
        self.vector.get(id.0).expect("Invalid ID").borrow()
    }

    pub(crate) fn get_mut(&self, id: &Id) -> RefMut<T> {
        self.vector.get(id.0).expect("Invalid ID").borrow_mut()
    }

    pub(crate) fn insert(&mut self, value: T) -> Id {
        let id = Id(self.len());
        self.vector.push(RefCell::new(value));
        return id
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (Id, &RefCell<T>)> {
        self.vector.iter().enumerate().map(|(i, v)| (Id(i), v))
    }

    pub(crate) fn ids(&self) -> Vec<Id> {
        self.vector.iter().enumerate().map(|(i, _v)| Id(i)).collect()
    }

}