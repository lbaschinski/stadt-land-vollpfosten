use std::{thread, time::Duration};
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

pub fn start_timer(timeout: u32) {
    if timeout > 999 {
        panic!("Please provide a timeout lower than 999s!");
    }
    println!("Timer:");
    let mut stdout = stdout();

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
