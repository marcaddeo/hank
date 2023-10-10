use anyhow::{bail, Result};
use confique::{yaml::FormatOptions, Config};
use std::path::PathBuf;

#[derive(Clone, Debug, Config)]
pub struct Conf {
    /// A list of absolute paths to plugin .wasm files to load.
    pub plugins: Vec<PathBuf>,
}

impl Conf {
    pub fn load(config_path: Option<PathBuf>) -> Result<Conf> {
        let config_path = get_config_path(config_path)?;
        let config = Conf::builder().env().file(config_path).load()?;

        Ok(config)
    }
}

pub fn get_config_template() -> String {
    confique::yaml::template::<Conf>(FormatOptions::default())
}

pub fn print_config_template() {
    println!("{}", get_config_template());
}

pub fn get_config_path(config_path: Option<PathBuf>) -> Result<PathBuf> {
    match config_path {
        Some(path) => Ok(path),
        None => {
            let xdg_dirs = xdg::BaseDirectories::with_prefix("hank")?;
            Ok(xdg_dirs.get_config_file("config.yml"))
        }
    }
}

pub fn write_config_template(config_path: Option<PathBuf>) -> Result<PathBuf> {
    let config_path = get_config_path(config_path)?;
    let config_template = get_config_template();

    let config_path_dir = match config_path.parent() {
        Some(s) => s,
        None => bail!("Could not determine config file parent dir"),
    };

    std::fs::create_dir_all(config_path_dir)?;
    std::fs::write(config_path.clone(), config_template)?;

    Ok(config_path)
}
