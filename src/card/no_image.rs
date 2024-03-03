use iced::{
    widget::{column, text},
    Element,
};

use crate::Message;
#[derive(Debug, Clone)]
pub struct NoImageCard {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
}

impl NoImageCard {
    pub fn view(&self) -> Element<Message> {
        column!(text(self.name.clone()), text(self.cmc.unwrap_or(0.0))).into()
    }
}
