//! Configuration from `persist.vendor.go7fl.*`

/// Paths and property names
pub struct Config {
    /// i2c-dev adapter with the LM3630A (`persist.vendor.go7fl.i2c_dev`)
    pub i2c_dev: String,

    /// Brightness signal file, 0..=255 (`persist.vendor.go7fl.br_file`)
    pub br_file: String,

    /// Node to write `1` to when the chip doesn't answer
    pub kick_path: String,
}

/// Kick property
pub const KICK_PROP: &str = "persist.vendor.go7fl.kick";

/// Warmth property
pub const CT_PROP: &str = "persist.vendor.go7fl.ct";

/// I2C adapter property
pub const I2C_DEV_PROP: &str = "persist.vendor.go7fl.i2c_dev";

/// Brightness signal file property
pub const BR_FILE_PROP: &str = "persist.vendor.go7fl.br_file";

impl Config {
    /// Read the config through `get`, falling back to defaults
    #[must_use]
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let string = |name: &str, default: &str| {
            get(name)
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| default.to_owned())
        };
        let kick = get(KICK_PROP);
        Self {
            i2c_dev: string(I2C_DEV_PROP, "/dev/i2c-3"),
            br_file: string(BR_FILE_PROP, "/sys/class/backlight/panel0-backlight/brightness"),
            kick_path: match kick.as_deref() {
                Some("none") => String::new(),
                Some(v) if !v.is_empty() => v.to_owned(),
                _ => "/sys/class/backlight/onyx_bl_br/brightness".to_owned(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn read(props: &[(&str, &str)]) -> Config {
        let props: HashMap<_, _> = props.iter().copied().collect();
        Config::read(|name| props.get(name).map(|v| (*v).to_owned()))
    }

    #[test]
    fn defaults() {
        let c = read(&[]);
        assert_eq!(c.i2c_dev, "/dev/i2c-3");
        assert_eq!(c.br_file, "/sys/class/backlight/panel0-backlight/brightness");
    }

    #[test]
    fn reads_overrides() {
        let c = read(&[(I2C_DEV_PROP, "/dev/i2c-2"), (BR_FILE_PROP, "/dev/x/br")]);
        assert_eq!(c.i2c_dev, "/dev/i2c-2");
        assert_eq!(c.br_file, "/dev/x/br");
    }

    #[test]
    fn kick_defaults_and_disables() {
        assert_eq!(read(&[]).kick_path, "/sys/class/backlight/onyx_bl_br/brightness");
        assert_eq!(read(&[(KICK_PROP, "none")]).kick_path, "");
        assert_eq!(read(&[(KICK_PROP, "/dev/x/kick")]).kick_path, "/dev/x/kick");
    }

    #[test]
    fn empty_prop_falls_back() {
        let c = read(&[(I2C_DEV_PROP, "")]);
        assert_eq!(c.i2c_dev, "/dev/i2c-3");
    }
}
