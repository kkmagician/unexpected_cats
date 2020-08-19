mod models;

use std::env;
use std::fs::File;
use std::io::Read;

use reqwest::blocking::{Client, Response};
use reqwest::Error as R_Error;
use serde_json::{from_value, Value, Error};

use crate::models::vk::WallPost;
use crate::models::tg::TgMediaGroup;

fn read_key_secret(key: &str) -> Option<String> {
    let mut value = String::new();
    let fs = File::open(format!("/run/secrets/{}", key))
        .and_then(|mut f| f.read_to_string(&mut value));

    match fs {
        Ok(_) => Some(value.clone()),
        Err(_) => None
    }
}

fn read_key_env(key: &str) -> Option<String> {
    env::var_os(key).and_then(|v| v.into_string().ok())
}

fn find_key(key: &str) -> Option<String> {
    read_key_env(key).or_else(|| read_key_secret(key))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vk_token = find_key("VK_TOKEN").unwrap_or(String::from(""));
    let tg_token = find_key("TG_TOKEN").unwrap_or(String::from(""));
    let tg_api_url = format!("https://api.telegram.org/bot{}/sendMediaGroup", tg_token);

    let params = [
        ("owner_id", "-97665403"),
        ("count", "10"),
        ("v", "5.122"),
        ("access_token", &vk_token)
    ];

    let client = reqwest::blocking::Client::new();
    let res = client.get("https://api.vk.com/method/wall.get")
        .query(&params)
        .send()?
        .json::<Value>()?;

    let posts = parse_posts(&res)?;

    match posts {
        Some(v) => v.iter()
            .filter_map(|post| post.to_message())
            .map(|msg| msg.to_media_group("@bullytest".to_string()) )
            .map(|msg| send_tg_message(&client, &tg_api_url, &msg))
            .for_each(|res| println!("Result: {:#?}", res)),

        None => println!("Could not parse posts")
    };

    Ok(())
}

fn send_tg_message(client: &Client, api_url: &str, message: &TgMediaGroup)
    -> Result<Response, R_Error> {
    client.post(api_url).json(message).send()
}

fn parse_posts(json: &Value) -> Result<Option<Vec<WallPost>>, Error> {
    let items = json.get("response").and_then(|j| j.get("items"));
    items.map(|v| from_value::<Vec<WallPost>>(v.clone()))
        .map_or(Ok(None), |r| r.map(Some))
}
