use bytes::Bytes;
use iced::{
    widget::{column, image::Handle, row, text, Image},
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
    pub oracle_text: String,
}

impl NormalCard {
    pub fn view(&self) -> Element<Message> {
        if let Some(image) = &self.image {
            column!(Image::new(Handle::from_memory(image.clone()))
                .content_fit(iced::ContentFit::Contain),)
        } else {
            column!(
                row!(text("Name:"), text(&self.name),),
                row!(
                    text("cmc:"),
                    text(
                        self.cmc
                            .map(|cmc| cmc.to_string())
                            .unwrap_or("".to_string())
                    )
                ),
                text(&self.oracle_text)
            )
            .align_items(iced::Alignment::Start)
        }
        .into()
    }
}
