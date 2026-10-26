//! Frontlight daemon for the Boox Go 7

mod i2c;
mod log;
mod settings;
mod sysprop;
mod tables;
mod tables_data;

use crate::log::{alogi, alogw};
use std::time::Duration;

/// How often the signals are re-read (polling loop)
const TICK: Duration = Duration::from_millis(250);

fn main() {
    let config = settings::Config::read(sysprop::get);
    log::set_debug(
        sysprop::get("persist.vendor.go7fl.debug")
            .and_then(|v| sysprop::parse_int(&v))
            .is_some_and(|v| v != 0),
    );
    run(&config);
}

/// Claim the chip, then apply frontlight state forever
fn run(config: &settings::Config) -> ! {
    let chip = loop {
        if let Some(chip) = i2c::Chip::open(&config.i2c_dev) {
            break chip;
        }
        alogw!("{} not available yet, retrying every 1s", config.i2c_dev);
        std::thread::sleep(Duration::from_secs(1));
    };
    alogi!(
        "frontlightd started: {} @ 0x38, signal {}",
        config.i2c_dev,
        config.br_file
    );

    let mut applied: Option<(u8, u8)> = None;
    loop {
        std::thread::sleep(TICK);
        let state = (brightness(&config.br_file), warmth());
        if applied == Some(state) {
            continue;
        }
        let codes = tables::codes(state.0, state.1);
        let mut ok = chip.write(&codes);
        if !ok && !config.kick_path.is_empty() && state.0 != 0 {
            // The chip answers nothing while unpowered
            kick(&config.kick_path);
            ok = chip.write(&codes);
        }
        if ok {
            if log::debug() {
                alogi!("br={} warmth={}", state.0, state.1);
            }
            applied = Some(state);
        }
    }
}

/// Asks the Onyx driver to raise the frontlight power rail (stock kernel only!)
fn kick(path: &str) {
    if let Err(e) = std::fs::write(path, b"1\n") {
        alogw!("kick {path}: {e}");
    }
}

/// Brightness level from signal file's 0..=255 value
fn brightness(path: &str) -> u8 {
    let v = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| text.split_whitespace().next()?.parse::<u8>().ok())
        .unwrap_or(0);
    tables::level(v)
}

/// Warmth level from the property, clamped to 0..=32, default 0 (cold)
fn warmth() -> u8 {
    let v = sysprop::get(settings::CT_PROP)
        .and_then(|v| sysprop::parse_int(&v))
        .unwrap_or(0);
    u8::try_from(v.clamp(0, i32::from(tables::MAX))).unwrap_or(0)
}
