use std::thread;
use std::time::Duration as dur;
use std::process::Command;
use std::io::{self, Write};
use chrono::{Local, Duration};
use std::net::TcpStream;
use std::env;
use std::io::BufRead;
use regex::Regex;
use std::fmt;

type NetworkCheckFn = fn() -> bool;

fn check_network_connection_ping() -> bool {
    // Execute the ping command
    let output = Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg("8.8.8.8")
        .output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn check_network_connection_tcp() -> bool {
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

const COL_RED: &str = "\x1b[31m";
const COL_RESET: &str = "\x1b[0m";
const COL_GREEN: &str = "\x1b[32m";
const OK: &str = "\x1b[32mOK\x1b[0m";
const FAIL: &str = "\x1b[31mFAIL\x1b[0m";

#[derive(PartialEq)]
//#[derive(Debug)]
enum Status {
    OK,
    Timeout,
    Unknown,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::OK => write!(f, "{}OK{}", COL_GREEN, COL_RESET),
            Status::Timeout => write!(f, "{}ERR{}", COL_RED, COL_RESET),
            Status::Unknown => write!(f, "Unknown"),
        }
    }
}

struct RunningStatus {
    ping_ok: Regex,
    request_timeout: Regex,
    status: Status,
    since: String,
    count: i32,
}

impl RunningStatus {


    fn new() -> Self {
        RunningStatus {
            ping_ok: Regex::new(r"(?P<time>\d{2}:\d{2}:\d{2})\.\d+ \d+ bytes from \d+.\d+.\d+.\d+: icmp_seq=\d+ ttl=\d+ time=(?P<speed>.*)").unwrap(),
            request_timeout: Regex::new(r"(?P<time>\d{2}:\d{2}:\d{2})\.\d+ Request timeout for icmp_seq \d+").unwrap(),
            status: Status::Unknown,
            since: "beginning".to_owned(),
            count: 0,
        }
    }

    fn register_line(&mut self, line: String) {

        let mut new_status = Status::Unknown; 
        let mut since = "now";
        let mut extra = "";
        if let Some(caps) = self.ping_ok.captures(&line) {
            new_status = Status::OK;
            since = caps.name("time").map_or("", |m| m.as_str());
            extra = caps.name("speed").map_or("", |m| m.as_str());
        } else if let Some(caps) = self.request_timeout.captures(&line) {
            new_status = Status::Timeout;
            since = caps.name("time").map_or("", |m| m.as_str());
        } else {
            println!("{} {}", FAIL, line)
        }
        if new_status != self.status {
            self.status = new_status;
            self.since = since.to_owned();
            self.count = 0;
            println!()
        }
        self.count += 1;
        print!("\r{} x {} since {} {}\x1b[K", self.status, self.count, self.since, extra);

        io::stdout().flush().unwrap();
    }
}

fn main() {

    let stdin = io::stdin();

    // Lock the standard input handle and create a buffered reader
    let handle = stdin.lock();
    let mut lines = handle.lines();
    let mut lines_processed = 0;
    let mut accumulator = RunningStatus::new();

    // Read and print each line
    while let Some(line) = lines.next() {
        lines_processed += 1;
        match line {
            Ok(content) => accumulator.register_line(content),
            Err(error) => eprintln!("Error reading line: {}", error),
        }
    }

    println!("\nDone, processed {} lines", lines_processed);

    let args: Vec<String> = env::args().collect();

    // Define the running mode based on the first argument
    let running_mode = if args.len() > 1 {
        &args[1]
    } else {
        return
    };

    let network_check: NetworkCheckFn = if running_mode == "ping" {
        println!("using ping");
        check_network_connection_ping
    } else {
        println!("using tcp::connect");
        check_network_connection_tcp
    };

    let mut seconds_elapsed: i64 = 0;
    let mut last_status = "initial";
    loop {
        let current_status = if network_check() {OK} else {FAIL};
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
