use std::{fs::File, sync::{LazyLock, Mutex, MutexGuard}, io::Write};

static LOG: LazyLock<Mutex<Log>> = LazyLock::new(|| Mutex::new(Log::new()));

pub(crate) fn log() -> MutexGuard<'static, Log> {
    LOG.lock().unwrap()
}

pub(crate) struct Log {
    file: File
}

impl Log {

    fn new() -> Self {
        let f = File::create("logs.log").unwrap();
        Self {
            file: f
        }
    }

    pub(crate) fn log(&mut self, tag: &str, line: &str) {
        writeln!(&mut self.file, "[{tag}] {line}").unwrap();
    }

}

#[macro_export]
macro_rules! history_trace {
    ($($arg:tt)*) => {{
        // $crate::engine::debug::log::log().log("HISTORY", &format!($($arg)*));
    }};
}


#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        $crate::engine::debug::log::log().log("WARN", &format!($($arg)*));
    }};
}