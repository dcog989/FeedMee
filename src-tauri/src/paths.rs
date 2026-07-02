use std::path::PathBuf;

fn home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
}

pub fn config_dir() -> PathBuf {
    home().join(".config/com.feedmee.app")
}

pub fn local_data_dir() -> PathBuf {
    home().join(".local/share/com.feedmee.app")
}
