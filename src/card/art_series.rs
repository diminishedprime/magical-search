use iced::{
    widget::{column, image::Handle, text, Image},
    Element,
};

use crate::Message;

#[derive(Debug, Clone)]
pub struct ArtSeries {
    pub id: String,
    pub name: String,
    pub selected_face: usize,
    pub face: Option<Vec<u8>>,
    pub num_faces: usize,
}

impl ArtSeries {
    pub fn view(&self) -> Element<Message> {
        match &self.face {
            Some(image) => {
                column!(Image::new(Handle::from_memory(image.clone())))
            }
            None => column!(text("No image for this face.")),
        }
        .into()
    }
}
