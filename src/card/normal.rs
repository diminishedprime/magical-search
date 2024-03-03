use iced::{
    widget::{column, image::Handle, Image},
    Element,
};

use crate::Message;

#[derive(Debug, Clone)]
pub struct NormalCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub image: Vec<u8>,
}

impl NormalCard {
    pub fn view(&self) -> Element<Message> {
        column!(Image::new(Handle::from_memory(self.image.clone()))
            .content_fit(iced::ContentFit::Contain),)
        .into()
    }
}
