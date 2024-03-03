use bytes::Bytes;
use iced::{
    widget::{button, column, image::Handle, text, Image},
    Element,
};

use crate::Message;

#[derive(Debug, Clone)]
pub struct ArtSeries {
    pub id: String,
    pub name: String,
    pub selected_face: usize,
    pub face: Option<Bytes>,
    pub num_faces: usize,
}

impl ArtSeries {
    pub fn view(&self) -> Element<Message> {
        match &self.face {
            Some(image) => {
                column!(
                    Image::new(Handle::from_memory(image.clone())),
                    button("Next face").on_press(Message::NextFace {
                        card_id: self.id.clone()
                    }),
                )
            }
            None => column!(text("No image for this face.")),
        }
        .into()
    }
}
