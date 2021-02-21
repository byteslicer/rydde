use indicatif::ProgressBar;
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::fmt;

pub struct ProcessResult {
    pub copied: AtomicU64,
    pub exist: AtomicU64,
    pub unknown: AtomicU64
}

impl ProcessResult {
    pub fn new() -> ProcessResult {
        ProcessResult {
            copied: AtomicU64::new(0),
            exist: AtomicU64::new(0),
            unknown: AtomicU64::new(0),
        }
    }

    pub fn inc_copied(&self) {
        self.copied.fetch_add(1, Ordering::SeqCst);
    }

    pub fn inc_exist(&self) {
        self.exist.fetch_add(1, Ordering::SeqCst);
    }

    pub fn inc_unknown(&self) {
        self.unknown.fetch_add(1, Ordering::SeqCst);
    }
}

impl fmt::Display for ProcessResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Copied files: {}", self.copied.load(Ordering::SeqCst))?;
        writeln!(f, "Already existing files: {}", self.exist.load(Ordering::SeqCst))?;
        writeln!(f, "Unknown files: {}", self.unknown.load(Ordering::SeqCst))
    }
}

pub struct Context {
    pub buffer: Cursor<Vec<u8>>,
    pub bar: ProgressBar,
    pub result: Arc<ProcessResult>
}

impl Context {
    pub fn new(bar: &ProgressBar, result: &Arc<ProcessResult>) -> Context {
        Context {
            buffer: Cursor::new(Vec::with_capacity(1024)),
            bar: bar.clone(),
            result: result.clone()
        }
    }    
}