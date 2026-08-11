use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Config {
    pub(crate) id_strategy: IdStrategy,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            id_strategy: IdStrategy::Timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdStrategy {
    Timestamp,
    Uid,
    Uuid,
}

impl IdStrategy {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "timestamp" => Some(Self::Timestamp),
            "uid" => Some(Self::Uid),
            "uuid" => Some(Self::Uuid),
            _ => None,
        }
    }
}

pub(crate) fn load_config(path: &Path) -> io::Result<Config> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(err) => return Err(err),
    };

    parse_config(&contents).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

pub(crate) fn parse_config(contents: &str) -> Result<Config, String> {
    let mut config = Config::default();

    for (line_index, line) in contents.lines().enumerate() {
        let line = line
            .split_once('#')
            .map_or(line, |(before, _)| before)
            .trim();
        if line.is_empty() {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {}: expected `key = value`", line_index + 1))?;
        let key = key.trim();
        let value = value.trim().trim_matches('"');

        match key {
            "id" | "id_strategy" => {
                config.id_strategy = IdStrategy::parse(value).ok_or_else(|| {
                    format!("line {}: unknown id strategy `{value}`", line_index + 1)
                })?;
            }
            _ => {
                return Err(format!(
                    "line {}: unknown config key `{key}`",
                    line_index + 1
                ))
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("todolog-{name}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn parses_default_config() {
        assert_eq!(parse_config("").unwrap(), Config::default());
    }

    #[test]
    fn parses_configured_id_strategies() {
        assert_eq!(
            parse_config("id = uid").unwrap().id_strategy,
            IdStrategy::Uid
        );
        assert_eq!(
            parse_config("id = timestamp").unwrap().id_strategy,
            IdStrategy::Timestamp
        );
        assert_eq!(
            parse_config("id_strategy = \"uuid\" # stable UUID-shaped IDs").unwrap(),
            Config {
                id_strategy: IdStrategy::Uuid
            }
        );
    }

    #[test]
    fn rejects_unknown_config_keys_and_id_strategies() {
        assert!(parse_config("format = markdown")
            .unwrap_err()
            .contains("unknown config key"));
        assert!(parse_config("id = snowflake")
            .unwrap_err()
            .contains("unknown id strategy"));
    }

    #[test]
    fn load_config_uses_default_when_file_is_missing() {
        let dir = temp_dir("missing-config");
        fs::create_dir_all(&dir).unwrap();
        let config = load_config(&dir.join(".todolog")).unwrap();

        assert_eq!(config, Config::default());

        let _ = fs::remove_dir_all(&dir);
    }
}
