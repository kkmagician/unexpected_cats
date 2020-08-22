mod models;

use std::env;
use std::fs::File;
use std::io::Read;

use redis::Commands;
use reqwest::blocking::Client;
use serde_json::{from_value, Value, Error};

use crate::models::vk::WallPost;
use crate::models::tg::TgMessage;

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

fn make_pair(pair: &str) -> (String, String, u8) {
    let mut split = pair.split(':');
    let owner = split.next().unwrap_or("");
    let chat = split.next().unwrap_or("");
    let posts_count: u8 = split.next().and_then(|cnt| cnt.parse().ok()).unwrap_or(5);

    (String::from(owner), String::from(chat), posts_count)
}

fn get_pairs(con: &mut redis::Connection) -> Vec<(String, String, u8)> {
    let default: Vec<(String, String, u8)> = vec![];
    let pairs_raw: Result<Vec<String>, redis::RedisError> = con.lrange("owner:chat", 0, -1);

    match pairs_raw {
        Ok(ps) => ps.iter()
            .map(|s| make_pair(&s) )
            .collect(),
        Err(_) => default
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //Keys
    let vk_token = find_key("VK_TOKEN").unwrap_or(String::from(""));
    let tg_token = find_key("TG_TOKEN").unwrap_or(String::from(""));
    let tg_api_url = format!("https://api.telegram.org/bot{}/sendMediaGroup", tg_token);

    // Redis
    let redis_host = read_key_env("REDIS_HOST").unwrap_or(String::from("127.0.0.1"));
    let redis_port = read_key_env("REDIS_PORT")
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(6379);
    let redis_db = read_key_env("REDIS_DB")
        .and_then(|db| db.parse::<u8>().ok())
        .unwrap_or(0);
    let redis_pass = find_key("REDIS_PASS");

    let redis_connection_info = redis::ConnectionInfo {
        addr: Box::from(redis::ConnectionAddr::Tcp(redis_host, redis_port)),
        db: redis_db as i64,
        username: None,
        passwd: redis_pass
    };

    // Clients
    let client = reqwest::blocking::Client::new();
    let r_client = redis::Client::open(redis_connection_info)?;
    let mut r = r_client.get_connection()?;

    let pairs = get_pairs(&mut r);
    println!("Found {} pairs", pairs.len());

    for (owner, chat, posts_count) in pairs.iter() {
        println!("Fetching {} posts of {} for {}", posts_count, owner, chat);
        let owner_key = format!("owner:{}:{}", owner, chat);
        let latest_post: Option<i32> = r.get(&owner_key)?;
        let latest_post: i32 = latest_post.unwrap_or(0);

        let posts: Vec<TgMessage> = get_vk_posts(&client, owner, &vk_token, posts_count)
            .iter().rev()
            .filter_map(|post| {
                if post.date > latest_post { post.to_message() } else { None }
            }).collect();

        println!("{} new posts in {} for {}", posts.len(), owner, chat);
        for post in posts {
            match post.send_tg_message(&client, chat, &tg_api_url) {
                Ok(date) => {
                    println!("{:?}", post);
                    r.set(&owner_key, date)?
                },
                Err(e) => {
                    println!("{:?}", e);
                    break
                }
            }
        }
    }

    Ok(())
}

fn get_vk_posts(client: &Client, group_id: &str, access_token: &str, posts_count: &u8)
    -> Vec<WallPost> {
    let params = [
        ("owner_id", group_id),
        ("count", &posts_count.to_string()),
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

fn parse_posts(json: &Value) -> Result<Option<Vec<WallPost>>, Error> {
    let items = json.get("response").and_then(|j| j.get("items"));
    items.map(|v| from_value::<Vec<WallPost>>(v.clone()))
        .map_or(Ok(None), |r| r.map(Some))
}
