extern crate reqwest;
extern crate toml;

use std::fs;
use std::io::Read;

#[derive(Debug)]
struct Config {
    source_blog: String,
    destination_blog: String,
    consumer_key: String,
    secret_key: String,
}

fn parse_config(filename: &str) -> Config {
    let mut config_file = fs::OpenOptions::new()
        .read(true).open(filename).expect("couldn't open config file?!");
    let mut config_content_buffer = String::new();
    config_file.read_to_string(&mut config_content_buffer)
        .expect("couldn't read config file?!");
    let config_value = &config_content_buffer.parse::<toml::Value>().unwrap();
    let config_table = config_value.get("tumblr_steam_shovel").unwrap();
    let source_blog = config_table.get("source_blog")
        .unwrap().as_str().unwrap().to_owned();
    let destination_blog = config_table.get("destination_blog")
        .unwrap().as_str().unwrap().to_owned();
    let consumer_key = config_table.get("consumer_key")
        .unwrap().as_str().unwrap().to_owned();
    let secret_key = config_table.get("secret_key")
        .unwrap().as_str().unwrap().to_owned();
    Config { source_blog, destination_blog, consumer_key, secret_key }
}

fn main() {
    let config = parse_config("shovel.toml");
    println!("{:#?}", config);
}
