use std::error::Error;
use std::{thread, time};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug)]
pub struct TgMessage {
    date: i32,
    text: Option<String>,
    media: Vec<String>
}

impl TgMessage {
    pub fn new(date: i32, text: Option<String>, media: Vec<String>) -> TgMessage {
        TgMessage { date, text, media }
    }

    fn to_media_group(&self, chat_id: &str) -> TgMediaGroup {
        let photos: Vec<TgMedia> = self.media.iter().enumerate().map(|(i, media)| {
           if i == 0 {
               TgMedia {
                   typ: String::from("photo"),
                   caption: self.text.clone(),
                   media: media.clone()
               }
           } else {
               TgMedia { typ: String::from("photo"), caption: None, media: media.clone() }
           }
        }).collect();

        TgMediaGroup { chat_id: chat_id.clone().to_string(), media: photos }
    }

    pub fn send_tg_message(&self, client: &Client, chat_id: &str, api_url: &str)
        -> Result<i32, Box<dyn Error>> {
        let message = self.to_media_group(chat_id);
        let resp = client.post(api_url).json(&message).send()?;

        let status_code = resp.status().as_u16();
        if status_code == 200 {
            Ok(self.date)
        } else if status_code == 429 {
            let err: Value = resp.json()?;
            let retry_after = err.get("parameters")
                .and_then(|v| v.get("retry_after"))
                .and_then(|i| i.as_i64())
                .unwrap_or(5);

            println!("Flood error, sleeping for {} secs", retry_after);
            let retry_duration = time::Duration::from_secs(retry_after as u64);
            thread::sleep(retry_duration);

            self.send_tg_message(client, chat_id, api_url)
        } else {
            println!("{:?}", resp);
            let err: Value = resp.json()?;
            Err(err.to_string())?
        }
    }
}

#[derive(Serialize, Debug)]
pub struct TgMediaGroup {
    chat_id: String,
    media: Vec<TgMedia>
}

#[derive(Serialize, Debug)]
pub struct TgMedia {
    #[serde(rename="type")]
    typ: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,

    media: String
}
