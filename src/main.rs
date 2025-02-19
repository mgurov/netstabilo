use std::thread;
use std::time::Duration as dur;
use std::net::TcpStream;
use std::io::{self, Write};
use chrono::{Local, Duration};

fn check_network_connection() -> bool {
    // Try to connect to a reliable server, e.g., Google's public DNS
    if let Ok(_) = TcpStream::connect("8.8.8.8:53") {
        return true;
    }
    false
}

fn now() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn format_duration(seconds: i64) -> String {
    let duration = Duration::seconds(seconds);    
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

const OK: &str = "\x1b[32mOK\x1b[0m";
const FAIL: &str = "\x1b[31mFAIL\x1b[0m";

fn main() {
    let mut seconds_elapsed: i64 = 0;
    let mut last_status = "initial";
    loop {
        let current_status = if check_network_connection() {OK} else {FAIL};
        if current_status != last_status {
            println!("\n{} {}", now(), current_status);
            last_status = current_status;
            seconds_elapsed = 0;
        }

        print!("\r{} {} {}", now(), current_status, format_duration(seconds_elapsed));

        io::stdout().flush().unwrap();

        thread::sleep(dur::from_secs(1));
        seconds_elapsed += 1;
    }
}
