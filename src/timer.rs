use std::{thread, time::Duration};
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

// Timeout struct for input validation
pub struct TimeoutFromString {
    // this is private, so that `new` must be used and value is validated
    value: u32,
}
impl TimeoutFromString {
    pub fn new(input: String) -> TimeoutFromString {
        let value: u32 = match input.trim().parse() {
            Ok(value) => {
                if value < 1 || value > 999 {
                    panic!("Please provide a positive timeout lower than 999s!");
                }
                value
            },
            Err(_) => {
                panic!("Please type a number!");
            }
        };
        TimeoutFromString { value }
    }
    pub fn value(&self) -> u32 {
        self.value
    }
}

pub fn start_timer(timeout_from_string: TimeoutFromString) {
    println!("Timer:");
    let mut stdout = stdout();
    let timeout = timeout_from_string.value();

    stdout.execute(cursor::Hide).unwrap();
    for s in 0..(timeout+1) {
        stdout.queue(cursor::SavePosition).unwrap();
        stdout.write_all(format!("{:>3} /{timeout:>3} seconds.", s).as_bytes()).unwrap();
        if s != timeout {
            stdout.queue(cursor::RestorePosition).unwrap();
            stdout.flush().unwrap();
            thread::sleep(Duration::from_secs(1));
            stdout.queue(cursor::RestorePosition).unwrap();
            stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
        }
    }
    stdout.execute(cursor::Show).unwrap();
    println!();
}
