use once_cell::sync::Lazy;
use std::fmt::{Display, Formatter};
use std::ops::DerefMut;
use std::sync::Mutex;
use std::time::Instant;

/// A simple structure for tracking time intervals.
struct Time {
    /// The initial Instant where time tracking began.
    init: Instant,
    /// The Instant of the previous time measurement.
    prev: Instant,
}

impl Time {
    fn new() -> Self {
        let now = Instant::now();

        Self {
            init: now,
            prev: now,
        }
    }
}

static TIME: Lazy<Mutex<Time>> = Lazy::new(|| Mutex::new(Time::new()));

/// Initializes the global time tracking instance.
/// This function should be called before using the `time` function.
pub fn time_init() {
    #[cfg(feature = "time")]
    {
        println!("{:>12}  {:>12} {:>12}", "pass", "since prev", "since init");
        Lazy::force(&TIME);
    }
}

/// Tracks and prints the time elapsed since the last call to `time` or `time_init`.
pub fn time(pass: &str) {
    #[cfg(feature = "time")]
    {
        let now = Instant::now();
        let mut time = TIME.lock().unwrap();
        let since_init = now.duration_since(time.init);
        let since_prev = now.duration_since(time.prev);
        time.prev = now;
        println!("{pass:>12}: {since_prev:>#12?} {since_init:>#12?}");
    }
}
