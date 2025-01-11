use std::collections::HashMap;

use super::id_vec::Id;

pub struct ResourceMap<I, V> where I: Id {
    vector: Vec<V>,
    map: HashMap<String, I>
}

impl<I, V> ResourceMap<I, V> where I: Id {

    pub fn new() -> ResourceMap<I, V> {
        ResourceMap { vector: vec!(), map: HashMap::new() }
    }

    pub fn add(&mut self, key: &str, value: V) -> I {
        let id = I::new(self.vector.len());
        self.vector.push(value);
        self.map.insert(String::from(key), id.clone());
        return id
    }

    pub fn get(&self, id: &I) -> &V {
        return self.vector.get(id.as_usize()).expect("Using ResourceMap should be safe to unwrap")
    }

    pub fn find(&self, key: &str) -> &V {
        return self.get(&self.id_of(key))
    }

    pub fn id_of(&self, key: &str) -> I {
        return self.map.get(key).expect(&format!("Resource {key} not found")).clone()
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

    pub struct Test {}

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

