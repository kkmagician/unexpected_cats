mod models;

use std::env;
use std::fs::File;
use std::io::Read;

use reqwest::blocking::{Client, Response};
use reqwest::Error as R_Error;
use serde_json::{from_value, Value, Error};

use crate::models::vk::WallPost;
use crate::models::tg::TgMediaGroup;
use redis::Commands;

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

fn make_pair(pair: &str) -> (String, String) {
    let mut split = pair.split(':');
    let owner = split.next().unwrap_or("");
    let chat = split.next().unwrap_or("");

    (String::from(owner), String::from(chat))
}

fn get_pairs(con: &mut redis::Connection) -> Vec<(String, String)> {
    let default: Vec<(String, String)> = vec![];
    let pairs_raw: Result<Vec<String>, redis::RedisError> = con.lrange("owner:chat", 0, -1);

    match pairs_raw {
        Ok(ps) => ps.iter()
            .map(|s| make_pair(&s) )
            .collect(),
        Err(_) => default
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vk_token = find_key("VK_TOKEN").unwrap_or(String::from(""));
    let tg_token = find_key("TG_TOKEN").unwrap_or(String::from(""));
    let tg_api_url = format!("https://api.telegram.org/bot{}/sendMediaGroup", tg_token);
    let redis_host = read_key_env("REDIS_HOST").unwrap_or(String::from("127.0.0.1"));
    let redis_port = read_key_env("REDIS_HOST").unwrap_or(String::from("6379"));
    let redis_db = read_key_env("REDIS_DB").unwrap_or(String::from("0"));
    let redis_pass = find_key("REDIS_PASS")
        .map(|pass| format!(":{}@", pass))
        .unwrap_or(String::new());

    let redis_url = format!("redis://{}{}:{}/{}", redis_pass, redis_host, redis_port, redis_db);

    let client = reqwest::blocking::Client::new();
    let r_client = redis::Client::open(redis_url)?;
    let mut r = r_client.get_connection()?;
    let pairs = get_pairs(&mut r);

    println!("{:?}", pairs);

    for (owner, chat) in pairs.iter() {
        let posts = get_vk_posts(&client, owner, &vk_token).iter()
            .filter_map(|post| post.to_message())
            .map(|msg| msg.to_media_group(chat.to_string()));
        println!("{}", chat.to_string());
    }

    // match posts {
    //     Some(v) => v.iter()
    //         .filter_map(|post| post.to_message())
    //         .map(|msg| msg.to_media_group("@bullytest".to_string()) )
    //         .map(|msg| send_tg_message(&client, &tg_api_url, &msg))
    //         .for_each(|res| println!("Result: {:#?}", res)),
    //
    //     None => println!("Could not parse posts")
    // };

    Ok(())
}

fn get_vk_posts(client: &Client, group_id: &str, access_token: &str) -> Vec<WallPost> {
    let params = [
        ("owner_id", group_id),
        ("count", "2"),
        ("v", "5.122"),
        ("access_token", access_token)
    ];

    let empty: Vec<WallPost> = vec![];
    client.get("https://api.vk.com/method/wall.get")
        .query(&params)
        .send().ok()
        .and_then(|resp| resp.json::<Value>().ok())
        .and_then(|v| parse_posts(&v).ok().flatten())
        .unwrap_or(empty)
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
