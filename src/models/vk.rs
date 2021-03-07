use serde::Deserialize;
use crate::models::tg::TgMessage;

#[derive(Deserialize, Debug, Clone)]
pub struct WallPost {
    pub date: i32,
    pub is_pinned: Option<u8>,
    text: Option<String>,
    attachments: Option<Vec<Attachment>>
}

impl WallPost {
    const MAX_TG_SIZE: usize = 1096;

    fn is_text_size_ok(&self) -> bool {
        let text_length = self.text.as_ref().map_or(0, |txt| txt.len());
        text_length <= WallPost::MAX_TG_SIZE
    }

    fn has_attachments(&self) -> bool {
        self.attachments.is_some()
    }

    fn has_no_links(&self) -> bool {
        self.text.as_ref()
            .map_or(true, |t| !(t.contains("http") || t.contains("vk.me")) )
    }

    pub fn is_ok_post(&self) -> bool {
        self.has_no_links() && self.is_text_size_ok() && self.has_attachments()
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Attachment {
    #[serde(rename="type")]
    typ: String,
    photo: Option<Photo>
}

#[derive(Deserialize, Debug, Clone)]
struct Photo {
    sizes: Vec<Size>
}

#[derive(Deserialize, Debug, Clone)]
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