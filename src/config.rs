use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    Ru,
    En,
}

impl Language {
    pub fn text(self, ru: &'static str, en: &'static str) -> &'static str {
        match self {
            Self::Ru => ru,
            Self::En => en,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Button {
    #[default]
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern {
    #[default]
    Single,
    Double,
    Triple,
    Burst,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Distribution {
    Uniform,
    #[default]
    Normal,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub language: Language,
    pub delay_ms: u64,
    pub hold_ms: u64,
    pub button: Button,
    pub pattern: Pattern,
    pub burst_count: u32,
    pub burst_interval_ms: u64,
    pub randomize: bool,
    pub distribution: Distribution,
    pub delay_jitter_ms: u64,
    pub hold_jitter_ms: u64,
    pub burst_jitter_ms: u64,
    pub breaks: bool,
    pub break_probability: u8,
    pub break_min_ms: u64,
    pub break_max_ms: u64,
    pub fixed_position: bool,
    pub x: i32,
    pub y: i32,
    pub radius_px: u32,
    pub hotkey: String,
    pub trigger_count: u8,
    pub trigger_timeout_ms: u64,
    pub start_delay_ms: u64,
    pub max_clicks: u64,
    pub max_duration_s: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            language: Language::Ru,
            delay_ms: 40,
            hold_ms: 10,
            button: Button::Left,
            pattern: Pattern::Single,
            burst_count: 3,
            burst_interval_ms: 45,
            randomize: true,
            distribution: Distribution::Normal,
            delay_jitter_ms: 15,
            hold_jitter_ms: 5,
            burst_jitter_ms: 0,
            breaks: false,
            break_probability: 5,
            break_min_ms: 200,
            break_max_ms: 1500,
            fixed_position: false,
            x: 0,
            y: 0,
            radius_px: 0,
            hotkey: "F6".into(),
            trigger_count: 1,
            trigger_timeout_ms: 400,
            start_delay_ms: 1500,
            max_clicks: 0,
            max_duration_s: 0,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!("Unsupported config version: {}", self.version);
        }
        for (name, value, min, max) in [
            ("delay_ms", self.delay_ms, 1, 3_600_000),
            ("hold_ms", self.hold_ms, 1, 60_000),
            ("burst_count", self.burst_count as u64, 1, 1000),
            ("burst_interval_ms", self.burst_interval_ms, 1, 60_000),
            ("delay_jitter_ms", self.delay_jitter_ms, 0, 60_000),
            ("hold_jitter_ms", self.hold_jitter_ms, 0, 60_000),
            ("burst_jitter_ms", self.burst_jitter_ms, 0, 60_000),
            ("break_probability", self.break_probability as u64, 0, 100),
            ("break_min_ms", self.break_min_ms, 1, 3_600_000),
            ("break_max_ms", self.break_max_ms, 1, 3_600_000),
            ("radius_px", self.radius_px as u64, 0, 1000),
            ("trigger_count", self.trigger_count as u64, 1, 5),
            ("trigger_timeout_ms", self.trigger_timeout_ms, 50, 5000),
            ("start_delay_ms", self.start_delay_ms, 0, 60_000),
            ("max_clicks", self.max_clicks, 0, 1_000_000_000),
            ("max_duration_s", self.max_duration_s, 0, 604_800),
        ] {
            if !(min..=max).contains(&value) {
                bail!("{name}: {value} is outside {min}..={max}");
            }
        }
        if self.break_min_ms > self.break_max_ms {
            bail!("break_min_ms must not exceed break_max_ms");
        }
        if self.x.unsigned_abs() > 100_000 || self.y.unsigned_abs() > 100_000 {
            bail!("Coordinates must be within -100000..=100000");
        }
        crate::hotkeys::parse_trigger(&self.hotkey)?;
        Ok(())
    }

    pub fn sequence_len(&self) -> u32 {
        match self.pattern {
            Pattern::Single => 1,
            Pattern::Double => 2,
            Pattern::Triple => 3,
            Pattern::Burst => self.burst_count,
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("io", "Lion", "LionAutoclicker")
        .context("Cannot locate the user configuration directory")?;
    Ok(dirs.config_dir().join("settings.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    let contents = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(error) => return Err(error).with_context(|| format!("Reading {}", path.display())),
    };
    let config: Config = toml::from_str(&contents).context("Invalid settings.toml")?;
    config.validate()?;
    Ok(config)
}

pub fn save(config: &Config) -> Result<()> {
    config.validate()?;
    let path = config_path()?;
    let parent = path.parent().context("Invalid configuration path")?;
    fs::create_dir_all(parent)?;
    // A same-directory temporary file allows an atomic replacement on all targets.
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(toml::to_string_pretty(config)?.as_bytes())?;
    temporary.as_file().sync_all()?;
    temporary.persist(&path).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_config_uses_defaults_and_round_trips() {
        let config: Config = toml::from_str("delay_ms = 123\nlanguage = 'En'").unwrap();
        config.validate().unwrap();
        assert_eq!(config.delay_ms, 123);
        assert_eq!(config.hold_ms, 10);
        let copy: Config = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
        assert_eq!(config, copy);
    }

    #[test]
    fn rejects_invalid_ranges_versions_and_unknown_fields() {
        for change in [
            "delay_ms = 0",
            "break_min_ms = 2000\nbreak_max_ms = 100",
            "version = 2",
            "trigger_count = 0",
            "hotkey = 'Ctrl+Shift+F12'",
        ] {
            let config: Config = toml::from_str(change).unwrap();
            assert!(config.validate().is_err(), "{change}");
        }
        assert!(toml::from_str::<Config>("delai_ms = 100").is_err());
        assert!(toml::from_str::<Config>("delay_ms = -1").is_err());
    }
}
