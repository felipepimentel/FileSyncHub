use anyhow::Result;
use filesynchub::config::Config;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_config_loading() -> Result<()> {
    let dir = tempdir()?;
    let config_path = dir.path().join("test_config.toml");

    let config_content = r#"
[general]
log_level = "info"

[[watch_dirs]]
path = "./test_dir"
recursive = true
include = ["*.txt", "*.md"]
exclude = ["*.tmp"]

[plugins.google_drive]
credentials_path = "./credentials/google_drive.json"
folder_id = "test_folder_id"
include = ["*.txt"]
exclude = ["*.tmp"]
"#;

    fs::write(&config_path, config_content)?;

    let config = Config::load(config_path.to_str().unwrap())?;

    assert_eq!(config.general.log_level, "info");
    assert_eq!(config.watch_dirs.len(), 1);
    assert!(config.plugins.google_drive.is_some());
    assert!(config.plugins.onedrive.is_none());

    Ok(())
}

#[test]
fn test_config_validation() -> Result<()> {
    let dir = tempdir()?;
    let config_path = dir.path().join("test_config.toml");
    let test_dir = dir.path().join("test_dir");
    fs::create_dir(&test_dir)?;

    let config_content = format!(
        r#"
[general]
log_level = "info"

[[watch_dirs]]
path = "{}"
recursive = true
include = ["*.txt"]
exclude = ["*.tmp"]
"#,
        test_dir.to_str().unwrap().replace('\\', "/")
    );

    fs::write(&config_path, config_content)?;

    let config = Config::load(config_path.to_str().unwrap())?;
    assert!(config.validate().is_ok());

    Ok(())
}

#[test]
fn test_invalid_watch_dir() -> Result<()> {
    let dir = tempdir()?;
    let config_path = dir.path().join("test_config.toml");

    let config_content = r#"
[general]
log_level = "info"

[[watch_dirs]]
path = "./nonexistent_dir"
recursive = true
"#;

    fs::write(&config_path, config_content)?;

    let config = Config::load(config_path.to_str().unwrap())?;
    assert!(config.validate().is_err());

    Ok(())
}
