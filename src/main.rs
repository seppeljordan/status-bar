use chrono::prelude::*;
use std::fmt;
use std::io;
use std::thread;
use std::time;

use battery::*;

mod battery;

#[derive(Debug)]
struct Clock(DateTime<Local>);

impl Clock {
    fn update(&mut self) -> Result<(), io::Error> {
        self.0 = Local::now();
        Ok(())
    }
}

#[derive(Debug)]
struct StatusBar {
    power_status: PowerStatus,
    clock: Clock,
}

impl StatusBar {
    #[inline]
    fn update(&mut self) -> Result<(), io::Error> {
        self.power_status.update()?;
        self.clock.update()
    }
}

impl fmt::Display for StatusBar {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} {}",
            self.power_status,
            self.clock.0.format("%Y-%m-%d %H:%M")
        )
    }
}

fn main() {
    let mut status = StatusBar {
        power_status: PowerStatus::read_from_sysfs().unwrap(),
        clock: Clock(Local::now()),
    };
    loop {
        println!("{}", status);
        thread::sleep(time::Duration::from_secs(3));
        status.update().unwrap();
    }
}
