use std::{collections::HashMap, slice::Iter};

use crate::commons::id_vec::Identified;

use super::id_vec::Id;

#[derive(Clone)]
pub(crate) struct ResourceMap<I, V> where I: Id {
    vector: Vec<V>,
    map: HashMap<String, I>
}

impl<I, V> ResourceMap<I, V> where I: Id {

    pub(crate) fn new() -> ResourceMap<I, V> {
        ResourceMap { vector: vec!(), map: HashMap::new() }
    }

    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.map.clear();
        self.vector.clear();
    }

    pub(crate) fn add(&mut self, key: &str, value: V) -> I {
        let id = I::new(self.vector.len());
        self.vector.push(value);
        self.map.insert(String::from(key), id.clone());
        return id
    }

    pub(crate) fn get<'a>(&'a self, id: &I) -> Identified<'a, I, V> {
        let value = self.vector.get(id.as_usize()).expect("Using ResourceMap should be safe to unwrap");
        return Identified::new(id.clone(), value);
    }

    pub(crate) fn try_get(&self, id: usize) -> Option<&V> {
        return self.vector.get(id)
    }

    pub(crate) fn find(&'_ self, key: &str) -> Identified<'_, I, V> {
        return self.get(&self.id_of(key))
    }

    pub(crate) fn id_of(&self, key: &str) -> I {
        return self.map.get(key).expect(&format!("Resource {key} not found")).clone()
    }

    pub(crate) fn validate_id(&self, id: usize) -> Option<I> {
        if id < self.vector.len() {
            return Some(I::new(id))
        } else {
            return None;
        }
    }

    pub(crate) fn iter(&'_ self) -> Iter<'_, V> {
        return self.vector.iter()
    }

}


#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
    struct TestId(usize);
    impl Id for TestId {
        fn new(id: usize) -> Self {
            TestId(id)
        }
        fn as_usize(&self) -> usize {
            self.0
        }
    }

    pub(crate) struct Test {}

    #[test]
    fn bench() {
        sub_bench(10000, 10);
        sub_bench(10000, 1000);
        sub_bench(10000, 10000);
    }

    fn sub_bench(i: usize, n: usize) {
        let mut map: ResourceMap<TestId, Test> = ResourceMap::new();
        
        for j in 0..n {
            map.add(&j.to_string(), Test{});
        }

        let now = Instant::now();
        for _ in 0..i {
            map.find("2");
        }
        let elapsed = now.elapsed();
        println!("find, n = {n}, i = {i}, t = {:.2?}", elapsed);
    }
}

