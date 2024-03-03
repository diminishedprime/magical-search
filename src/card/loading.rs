use iced::{
    widget::{column, text},
    Element,
};

use crate::Message;

#[derive(Debug, Clone)]
pub struct LoadingCard {
    pub id: String,
}

impl LoadingCard {
    pub fn view(&self) -> Element<Message> {
        column!(text("Loading card"),).into()
    }
}
