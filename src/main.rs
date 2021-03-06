#[macro_use] extern crate nickel;
#[macro_use] extern crate serde_derive;

mod ldap;
use ldap::Ldap;

mod server;
use server::Server;

extern crate toml;
use std::fs::File;
use std::io::prelude::*;

#[derive(Deserialize,Clone)]
pub struct DomainConfig {
    server: String,
    domain: String,
    secure: bool,
}

#[derive(Deserialize,Clone)]
pub struct WebConfig {
    ip: String,
    port: i64,
    session: i64,
}

#[derive(Deserialize,Clone)]
pub struct Config {
    domain: DomainConfig,
    web: WebConfig
}

fn main() {
    let config = load_config("./cfg/config.toml").expect("Unable to load config");
    let ad = Ldap::new(&config);
    let server = Server::new(ad,&config);
    server.start();
    
}

fn load_config (path: &str) -> Option<Config> {
    let file = File::open(path);
    let mut contents = String::new();
    if let Ok(mut file) = file {
        let _ = file.read_to_string(&mut contents);
        let config: Option<Config> = toml::from_str(&contents).ok();
        return config
    }

    None
}

