use std::time::{Duration, UNIX_EPOCH};

fn main() {
    let since_epoch = 5;
    let since_epoch = Duration::from_millis(since_epoch);
    let now = UNIX_EPOCH + since_epoch;
}
