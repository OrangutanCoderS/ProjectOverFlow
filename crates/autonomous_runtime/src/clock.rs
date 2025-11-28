use std::thread;
use std::time::Duration;

/// Clock abstraction so we can unit-test the runtime loop without real sleeps.
pub trait RuntimeClock {
    fn sleep(&mut self, duration: Duration);
}

/// Production clock: delegates to std::thread::sleep.
pub struct SystemClock;

impl RuntimeClock for SystemClock {
    fn sleep(&mut self, duration: Duration) {
        // Single-threaded, no cancellation logic.
        thread::sleep(duration);
    }
}

/// Test clock that *does not* actually sleep.
/// Useful in unit tests and benches.
#[derive(Default)]
pub struct NoopClock;

impl RuntimeClock for NoopClock {
    fn sleep(&mut self, _duration: Duration) {
        // Intentionally empty.
    }
}