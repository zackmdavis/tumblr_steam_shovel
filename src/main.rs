extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use std::error::Error;
use std::fs;
use std::io::Read;

use serde::Deserialize;


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

fn request_posts(config: Config, offset: usize) -> Result<Vec<Post>, Box<Error>> {
    let client = reqwest::Client::new()?;
    let url = format!(
        "https://api.tumblr.com/v2/blog/{}.tumblr.com/posts/text?api_key={}&tag={}&offset={}&filter=raw",
        config.source_blog,
        config.consumer_key,
        config.source_tag,
        offset
    );
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


fn main() {
    let config = parse_config("shovel.toml");
    let posts = request_posts(config, 0);
    println!("{:?}", posts);
}
