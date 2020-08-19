use serde::{Deserialize};
use crate::models::tg::TgMessage;

#[derive(Deserialize, Debug)]
pub struct WallPost {
    text: Option<String>,
    date: i32,
    attachments: Option<Vec<Attachment>>
}

#[derive(Deserialize, Debug)]
struct Attachment {
    #[serde(rename="type")]
    typ: String,
    photo: Option<Photo>
}

#[derive(Deserialize, Debug)]
struct Photo {
    sizes: Vec<Size>
}

#[derive(Deserialize, Debug)]
struct Size {
    url: String
}

impl WallPost {
    pub fn to_message(&self) -> Option<TgMessage> {
        let e: Vec<Attachment> = vec![];
        let attachments: &Vec<Attachment> = self.attachments.as_ref().unwrap_or(&e);
        let text: Option<String> = self.text.clone().and_then(|v| {
            if v.is_empty() { None } else { Some(v) }
        });
        let media: Vec<String> = attachments.iter()
            .filter_map(|att| {
                att.photo.as_ref()
                    .map_or(None, |p| {
                        p.sizes.last().map(|s| s.url.clone())
                    })
            }).collect();

        if media.is_empty() { None } else { Some(TgMessage::new(self.date, text, media)) }
    }
}