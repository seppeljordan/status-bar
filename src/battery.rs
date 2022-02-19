use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path;

#[derive(Debug, Copy, Clone)]
enum BatteryChargingStatus {
    Discharging,
    Charging,
    NotCharging,
}

#[derive(Debug)]
struct BatteryStatusBuilder {
    charging: Option<BatteryChargingStatus>,
    energy_full: Option<usize>,
    energy_now: Option<usize>,
}

impl BatteryStatusBuilder {
    #[inline]
    fn set_charging(&mut self, charging: BatteryChargingStatus) {
        self.charging = Some(charging);
    }
    #[inline]
    fn set_energy_full(&mut self, energy_full: usize) {
        self.energy_full = Some(energy_full);
    }
    #[inline]
    fn set_energy_now(&mut self, energy_now: usize) {
        self.energy_now = Some(energy_now);
    }
    #[inline]
    fn build(self) -> Option<BatteryStatus> {
        Some(BatteryStatus {
            charging: self.charging?,
            energy_full: self.energy_full?,
            energy_now: self.energy_now?,
        })
    }
}

impl Default for BatteryStatusBuilder {
    #[inline]
    fn default() -> Self {
        BatteryStatusBuilder {
            charging: None,
            energy_full: None,
            energy_now: None,
        }
    }
}

#[derive(Debug)]
struct BatteryStatus {
    charging: BatteryChargingStatus,
    energy_full: usize,
    energy_now: usize,
}

#[derive(Debug)]
pub struct PowerStatus {
    batteries: Vec<BatteryStatus>,
}

impl PowerStatus {
    pub fn read_from_sysfs() -> Result<Self, io::Error> {
        Ok(PowerStatus {
            batteries: get_batteries()?,
        })
    }

    #[inline]
    fn get_battery_percent(&self) -> Option<usize> {
        let energy_now: usize = self.batteries.iter().map(|b| b.energy_now).sum();
        let energy_full: usize = self.batteries.iter().map(|b| b.energy_full).sum();
        if energy_full == 0 {
            None
        } else {
            Some(100 * energy_now / energy_full)
        }
    }

    pub fn update(&mut self) -> Result<(), io::Error> {
        self.batteries = get_batteries()?;
        Ok(())
    }
}

impl fmt::Display for PowerStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.get_battery_percent() {
            None => write!(formatter, "No battery detected"),
            Some(battery) => write!(formatter, "Battery: {}%", battery),
        }
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
    let mut builder = BatteryStatusBuilder::default();
    for line in io::BufReader::new(file).lines() {
        if let [key, value] = line.ok()?.split('=').collect::<Vec<&str>>()[..] {
            match key {
                "POWER_SUPPLY_STATUS" => {
                    builder.set_charging(charging_status_from_string(value)?);
                }
                "POWER_SUPPLY_ENERGY_FULL" => {
                    builder.set_energy_full(value.parse().ok()?);
                }
                "POWER_SUPPLY_ENERGY_NOW" => {
                    builder.set_energy_now(value.parse().ok()?);
                }
                _ => (),
            }
        }
    }
    builder.build()
}

#[inline]
fn battery_from_sysfs_path(path: path::PathBuf) -> Result<Option<BatteryStatus>, io::Error> {
    let file = fs::File::open(path)?;
    Ok(battery_from_uevent_file(file))
}

#[inline]
fn get_battery_path(maybe_direntry: Result<fs::DirEntry, io::Error>) -> Option<path::PathBuf> {
    let mut directory_path = maybe_direntry.ok()?.path();
    {
        let filename = directory_path.file_name()?.to_str()?;
        if !filename.starts_with("BAT") {
            return None;
        }
    }
    directory_path.push("uevent");
    Some(directory_path)
}

fn get_batteries() -> Result<Vec<BatteryStatus>, io::Error> {
    fs::read_dir("/sys/class/power_supply")?
        .filter_map(get_battery_path)
        .map(battery_from_sysfs_path)
        .filter_map(Result::transpose)
        .collect::<Result<Vec<BatteryStatus>, io::Error>>()
}
