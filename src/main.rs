use chrono::prelude::*;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use std::thread;
use std::time;

#[derive(Debug)]
enum BatteryChargingStatus {
    Discharging,
    Charging,
    NotCharging,
}

#[derive(Debug)]
struct BatteryStatus {
    charging: BatteryChargingStatus,
    energy_now: usize,
    energy_full: usize,
}

#[derive(Debug)]
struct PowerStatus {
    batteries: Vec<BatteryStatus>,
}

impl PowerStatus {
    fn get_battery_percent(&self) -> usize {
        let energy_now: usize = self.batteries.iter().map(|b| b.energy_now).sum();
        let energy_full: usize = self.batteries.iter().map(|b| b.energy_full).sum();
        100 * energy_now / energy_full
    }

    fn update(&mut self) -> Result<(), io::Error> {
        self.batteries = get_batteries()?;
        Ok(())
    }
}

impl fmt::Display for PowerStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Battery: {}%", self.get_battery_percent())
    }
}

fn charging_status_from_string(value: &str) -> Option<BatteryChargingStatus> {
    match value {
        "Not charging" => Some(BatteryChargingStatus::NotCharging),
        "Discharging" => Some(BatteryChargingStatus::Discharging),
        "Charging" => Some(BatteryChargingStatus::Charging),
        _ => None,
    }
}

fn battery_from_uevent_file<R: io::Read>(file: R) -> Option<BatteryStatus> {
    let mut charging = None;
    let mut energy_now = None;
    let mut energy_full = None;
    for line in io::BufReader::new(file).lines() {
        match line.ok()?.split('=').collect::<Vec<&str>>()[..] {
            [key, value] => match key {
                "POWER_SUPPLY_STATUS" => {
                    charging = charging_status_from_string(value);
                }
                "POWER_SUPPLY_ENERGY_FULL" => {
                    energy_full = value.parse().ok();
                }
                "POWER_SUPPLY_ENERGY_NOW" => {
                    energy_now = value.parse().ok();
                }
                _ => (),
            },
            _ => (),
        }
    }
    Some(BatteryStatus {
        charging: charging?,
        energy_now: energy_now?,
        energy_full: energy_full?,
    })
}

fn get_batteries() -> Result<Vec<BatteryStatus>, io::Error> {
    let mut batteries = vec![];
    for directory in fs::read_dir("/sys/class/power_supply")? {
        let path = directory?.path();
        let path_str_repr = path.to_str().unwrap();
        if path_str_repr.starts_with("/sys/class/power_supply/BAT") {
            let file = fs::File::open(format!("{}/uevent", path_str_repr))?;
            match battery_from_uevent_file(file) {
                Some(battery) => batteries.push(battery),
                None => (),
            }
        }
    }
    Ok(batteries)
}

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

fn get_power_status() -> Result<PowerStatus, io::Error> {
    Ok(PowerStatus {
        batteries: get_batteries()?,
    })
}

fn main() {
    let mut status = StatusBar {
        power_status: get_power_status().unwrap(),
        clock: Clock(Local::now()),
    };
    loop {
        status.update().unwrap();
        println!("{}", status);
        thread::sleep(time::Duration::from_secs(3));
    }
}
