use serde::{Serialize};

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

    pub fn to_media_group(&self, chat_id: String) -> TgMediaGroup {
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

        TgMediaGroup { chat_id, media: photos }
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
