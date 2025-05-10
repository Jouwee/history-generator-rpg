use std::{collections::HashMap, sync::{LazyLock, Mutex, MutexGuard}, time::{Duration, Instant}};

static PERF: LazyLock<Mutex<Perf>> = LazyLock::new(|| Mutex::new(Perf::new()));

pub(crate) fn perf() -> MutexGuard<'static, Perf> {
    PERF.lock().unwrap()
}

pub(crate) struct Perf {
    open: HashMap<String, Instant>,
    last: HashMap<String, Duration>,
}

impl Perf {

    fn new() -> Self {
        Self {
            open: HashMap::new(),
            last: HashMap::new()
        }
    }

    pub(crate) fn start(&mut self, key: &str) {
        self.open.insert(String::from(key), Instant::now());
    }

    pub(crate) fn end(&mut self, key: &str) {
        let key = String::from(key);
        if let Some(open) = self.open.remove(&key) {
            self.last.insert(String::from(key), open.elapsed());
        }
    }

    pub(crate) fn debug_lines(&self) -> Vec<String> {
        let mut vec = Vec::new();
        for (key, time) in self.last.iter() {
            vec.push(format!("{}: {:.2?}", key, time));
        }
        return vec;
    }

}




