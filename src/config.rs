extern crate toml;

use common;

use std::env;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

pub type Config = HashMap<String, common::User>;

pub fn path(maybe_path: Option<&str>) -> PathBuf {
    maybe_path
        .map(PathBuf::from)
        .or(env::home_dir().map(|home| home.join(Path::new(".gus"))))
        .unwrap_or_default()
}

pub fn read(path: PathBuf) -> String {
    File::open(path)
        .map(|mut f| {
            let mut contents = String::new();
            f.read_to_string(&mut contents).expect("Can't read config file");
            contents
        })
        .expect("Can't open config file")
}

pub fn parse(config_data: String) -> Config {
    toml::from_str(config_data.as_ref())
        .expect("Can't parse config")
}

pub fn save(path: PathBuf, config: Config) {
    File::create(path)
        .map(|mut f| {
            let data = toml::to_string(&config).expect("Can't serialize config");
            f.write(data.as_ref()).expect("Can't write config file");;
            f.sync_all().expect("Can't sync config file to file system");
        })
        .expect("Can't create config file")
}