use std::fmt::Display;
use std::io::Write;
use std::str::FromStr;

use crate::StatusContext;
use crate::themes::Style;

#[derive(Debug)]
enum BatteryStatus {
    Charging {
        percent_charge: u8,
        time_remaining: Option<Duration>,
    },
    Discharging {
        percent_charge: u8,
        time_remaining: Option<Duration>,
    },
}

impl Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Charging {
                percent_charge,
                time_remaining: Some(duration),
            }
            | Self::Discharging {
                percent_charge,
                time_remaining: Some(duration),
            } => {
                write!(
                    f,
                    " {}% ({}h {}m) {}  ",
                    percent_charge,
                    duration.hour,
                    duration.min,
                    batt_icon(self)
                )
            }
            Self::Charging {
                percent_charge,
                time_remaining: None,
            }
            | Self::Discharging {
                percent_charge,
                time_remaining: None,
            } => {
                write!(f, " {}% {}  ", percent_charge, batt_icon(self))
            }
        }
    }
}

#[derive(Debug)]
struct Duration {
    pub hour: u64,
    pub min: u64,
}

enum DurationError {
    InvalidDurationString,
}

impl FromStr for Duration {
    type Err = DurationError;

    #[cfg(target_os = "macos")]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((h, m)) => {
                let hour = h
                    .parse::<u64>()
                    .map_err(|_| DurationError::InvalidDurationString)?;
                let min = m
                    .parse::<u64>()
                    .map_err(|_| DurationError::InvalidDurationString)?;
                Ok(Duration { hour, min })
            }
            None => Err(DurationError::InvalidDurationString),
        }
    }
}

pub fn batt_status(ctx: &StatusContext, buf: &mut impl Write) {
    if let Ok(status) = os_batt_status() {
        let percent_charge = match status {
            BatteryStatus::Charging {
                percent_charge,
                time_remaining: _,
            }
            | BatteryStatus::Discharging {
                percent_charge,
                time_remaining: _,
            } => percent_charge,
        };
        let batt_charge = match percent_charge {
            0..33 => Style::BatteryLow,
            33..66 => Style::BatteryMid,
            66..=u8::MAX => Style::BatteryHigh,
        };

        let _ = write!(
            buf,
            "{}",
            &ctx.theme.get_style_str(batt_charge, &status.to_string())
        );
    }
}

#[cfg(target_os = "macos")]
fn os_batt_status() -> Result<BatteryStatus, std::io::Error> {
    let status = duct::cmd!("pmset", "-g", "batt").read()?;
    let caps = match regex::Regex::new(r"(\d+)%;\s+([^;]+;)\s?(\d+:\d+)?") {
        Ok(it) => it,
        Err(_) => todo!(),
    }
    .captures(&status)
    .unwrap();

    let percent_text = caps.get(1).unwrap().as_str();
    let percent_charge = percent_text.parse::<u8>().unwrap();

    let time_remaining = match caps.get(3) {
        Some(m) => match Duration::from_str(m.as_str()) {
            Ok(d) => Some(d),
            Err(_) => None,
        },
        None => None,
    };

    let status_text = caps.get(2).unwrap().as_str();
    let batt_status = match &status_text[..3] {
        "dis" => BatteryStatus::Discharging {
            percent_charge,
            time_remaining,
        },
        _ => BatteryStatus::Charging {
            percent_charge,
            time_remaining,
        },
    };

    Ok(batt_status)
}

#[cfg(target_os = "linux")]
fn os_batt_status() -> Result<BatteryStatus, std::io::Error> {
    todo!()
}

fn batt_icon(status: &BatteryStatus) -> char {
    match status {
        BatteryStatus::Charging {
            percent_charge: _,
            time_remaining: _,
        } => nerdfonts::md::MD_POWER_PLUG,
        BatteryStatus::Discharging {
            percent_charge,
            time_remaining: _,
        } => match percent_charge / 10 {
            10 => nerdfonts::md::MD_BATTERY,
            9 => nerdfonts::md::MD_BATTERY_90,
            8 => nerdfonts::md::MD_BATTERY_80,
            7 => nerdfonts::md::MD_BATTERY_70,
            6 => nerdfonts::md::MD_BATTERY_60,
            5 => nerdfonts::md::MD_BATTERY_40,
            4 => nerdfonts::md::MD_BATTERY_40,
            3 => nerdfonts::md::MD_BATTERY_30,
            2 => nerdfonts::md::MD_BATTERY_20,
            1 => nerdfonts::md::MD_BATTERY_10,
            0 => nerdfonts::md::MD_BATTERY_10,
            11_u8..=u8::MAX => panic!("Unreasonable battery status!"),
        },
    }
}
