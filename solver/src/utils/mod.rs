pub mod chains;
pub mod svm;

pub use chains::*;
pub use svm::*;

/// Creates a slog Logger with standard timestamp and environment fields.
#[macro_export]
macro_rules! make_logger {
    ($drain:expr, $env:expr) => {
        slog::Logger::root(
            $drain,
            slog::o!(
                "timestamp" => slog::FnValue(|_| {
                    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                }),
                "environment" => $env
            ),
        )
    };
}

pub fn unix_timestamp_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}
