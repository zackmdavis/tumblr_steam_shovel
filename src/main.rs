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


// let consumer_key = console_input("input your consumer key:");
// let consumer_secret = console_input("input your consumer secret:");
// let consumer = Token::new(consumer_key, consumer_secret);

// let request = twitter::get_request_token(&consumer).unwrap();
// println!("open the following url:");
// println!("\t{}", twitter::get_authorize_url(&request));
// let pin = console_input("input pin:");
// let access = twitter::get_access_token(&consumer, &request, &pin).unwrap();

const REQUEST_TOKEN_URL: &str = "https://www.tumblr.com/oauth/request_token";
const AUTHORIZE_URL: &str = "https://www.tumblr.com/oauth/authorize";
const ACCESS_TOKEN_URL: &str = "https://www.tumblr.com/oauth/access_token";


// XXX: copied verbatim from
// https://github.com/gifnksm/twitter-api-rs/blob/ce497f38b57/src/lib.rs#L56-L65
//
// Augh! One cannot help but think that this functionality should already be
// built into oauth-client, saving naïve users much pain, while simultaneously
// fearing that the only possible response to this indictment is, "Patch or
// STFU"
fn split_query<'a>(query: &'a str) -> HashMap<Cow<'a, str>, Cow<'a, str>> {
    let mut param = HashMap::new();
    for q in query.split('&') {
        let mut s = q.splitn(2, '=');
        let k = s.next().unwrap();
        let v = s.next().unwrap();
        let _ = param.insert(k.into(), v.into());
    }
    param
}

fn get_request_token<'a>(consumer: &'a oauth_client::Token) -> oauth_client::Token<'a> {
    let response_bytes = oauth_client::get(
        REQUEST_TOKEN_URL, consumer, None, None).unwrap();
    let response = String::from_utf8(
        response_bytes).unwrap();
    let parameters = split_query(&response);
    oauth_client::Token::new(
        parameters.get("oauth_token").unwrap().to_string(),
        parameters.get("oauth_token_secret").unwrap().to_string()
    )
}

fn get_access_token<'a>() // -> oauth_client::Token<'a>
{
    // XXX TODO
}

fn auth(config: &Config) {
    let consumer = oauth_client::Token::new(
        config.consumer_key.clone(), config.secret_key.clone());
    let request_token = get_request_token(&consumer);

    // let access_token =

    // XXX TODO
}

impl Post {
    fn post(self, config: &Config) // -> oauth_client::Result<Vec<u8>>
    {
        // XXX: surely we should be able to save tokens between requests, but
        // I'm not very familiar with the OAuth protocol, this library seems
        // ... rough around the edges, and—going to be really honest here—this
        // program is kind of ad hoc, a half-day's exercise that the
        // shoemaker's children SHALL NOT go barefoot
        //
        // XXX: Cow::Cow::Cow::Cow;
        let request_token_url = "https://www.tumblr.com/oauth/request_token";
        let authorize_url = "https://www.tumblr.com/oauth/authorize";
        let access_token_url = "https://www.tumblr.com/oauth/access_token";
        let post_url = &format!("https://api.tumblr.com/v2/blog/{}/post",
                                config.destination_blog);

        // XXX what is OAuth

        // how do

        // let consumer = oauth_client::Token::new(
        //     config.consumer_key.clone(), config.secret_key.clone());
        // let mut request_parameters = oauth_client::ParamList::new();
        // request_parameters.insert(Cow::from("title"), Cow::from(self.title));
        // request_parameters.insert(Cow::from("date"), Cow::from(self.date));
        // request_parameters.insert(Cow::from("body"), Cow::from(self.body));
        // request_parameters.insert(Cow::from("tags"),
        //                           Cow::from(self.tags.join(",")));
        // oauth_client::post(
        //     request_token_url, &consumer, &consumer, Some(&request_parameters)
        // )

    }
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

fn main() {
    let config = parse_config("shovel.toml");
    let mut posts = request_posts(&config, 0).unwrap();

    println!("{:?}", posts);

    // XXX TODO

    // let post = posts.swap_remove(0);
    // let result = post.post(&config);
    // println!("{:#?}", result);
}
