#![allow(unused)]

extern crate oauth_client;
extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use serde_json::ser::Serializer;


#[derive(Debug)]
struct Config {
    source_blog: String,
    source_tag: String,
    tags_to_strip: Vec<String>,
    destination_blog: String,
    consumer_key: String,
    secret_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    title: String,
    date: String,
    body: String,
    tags: Vec<String>,
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
    let source_tag = config_table.get("source_tag")
        .unwrap().as_str().unwrap().to_owned();
    let tags_to_strip = config_table.get("tags_to_strip")
        .unwrap().as_array().unwrap().iter()
        .map(|t| t.as_str().unwrap().to_owned()).collect();
    let destination_blog = config_table.get("destination_blog")
        .unwrap().as_str().unwrap().to_owned();
    let consumer_key = config_table.get("consumer_key")
        .unwrap().as_str().unwrap().to_owned();
    let secret_key = config_table.get("secret_key")
        .unwrap().as_str().unwrap().to_owned();
    Config {
        source_blog, source_tag, tags_to_strip,
        destination_blog, consumer_key, secret_key
    }
}

fn request_posts(config: &Config, offset: usize) -> Result<Vec<Post>, Box<Error>> {
    let client = reqwest::Client::new()?;
    let url = format!(
        "https://api.tumblr.com/v2/blog/{}.tumblr.com/posts/text?api_key={}&tag={}&offset={}&filter=raw",
        config.source_blog,
        config.consumer_key,
        config.source_tag,
        offset
    );
    println!("requesting posts from {}", url);
    let payload: serde_json::Value = client
        .get(&url)?.send()?.error_for_status()?.json()?;
    let post_data: Vec<serde_json::Value> = payload
        .as_object().unwrap()
        .get("response").unwrap()
        .as_object().unwrap()
        .get("posts").unwrap()
        .as_array().unwrap()
        .to_vec();
    Ok(post_data.iter()
       .map(|p| {
           let mut post = Post::deserialize(p).unwrap();
           let destination_tags = post.tags.into_iter()
               .filter(|t| !config.tags_to_strip.contains(t)).collect();
           post.tags = destination_tags;
           post
       }).collect())
}

fn save_source_posts(config: &Config) {
    fs::create_dir("source_posts/");
    let mut i = 0;
    let mut offset = 0;
    loop {
        let posts = request_posts(&config, offset).unwrap();
        println!("we requested at offset {} and got {} posts back", offset, posts.len());
        if posts.is_empty() {
            break;
        }
        offset += posts.len();

        for post in posts {
            let mut file = fs::OpenOptions::new()
                .write(true).create(true)
                .open(format!("source_posts/{:03}.json", i)).unwrap();
            i += 1;
            let mut serializer = Serializer::new(file);
            post.serialize(&mut serializer);
        }
    }
}

fn post_to_destination(config: &Config) {
    let dir_listing = fs::read_dir("source_posts/").unwrap();
    for entry in dir_listing {
        let entry = entry.unwrap();
        let path = entry.path();
        println!("processing on-disk post {:?}", path);
        let mut file = fs::OpenOptions::new().read(true).open(path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer);
        let json: serde_json::Value = serde_json::from_str(&buffer).unwrap();
        let post = Post::deserialize(json);
        // ... ?
    }

}

fn main() {
    let config = parse_config("shovel.toml");
    save_source_posts(&config);
}
