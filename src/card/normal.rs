use bytes::Bytes;
use iced::{
    widget::{column, image::Handle, text, Image},
    Element,
};

use crate::Message;

#[derive(Debug, Clone)]
pub struct NormalCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub image: Option<Bytes>,
    pub image_uri: String,
}

impl NormalCard {
    pub fn view(&self) -> Element<Message> {
        if let Some(image) = &self.image {
            column!(Image::new(Handle::from_memory(image.clone()))
                .content_fit(iced::ContentFit::Contain),)
        } else {
            column!(
                text(&self.name),
                text(
                    self.cmc
                        .map(|cmc| cmc.to_string())
                        .unwrap_or("".to_string())
                )
            )
        }
        .into()
    }
}
