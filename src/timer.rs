use std::thread;
use std::time::Duration;

pub fn start_timer() {
    for s in 1..61 {
        thread::sleep(Duration::from_secs(1));
        println!("Timer:")
        println!("{s:>2}/60 seconds.")
    }
}
