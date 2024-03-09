use std::fmt::{self, Display, Formatter};

use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub uri: Option<String>,
    pub image: Option<Bytes>,
}

#[derive(Debug, Clone)]
pub struct CardData {
    pub id: String,
    pub name: String,
    pub cmc: Option<f64>,
    pub small_uri: Option<String>,
    pub normal_uri: Option<String>,
    pub large_uri: Option<String>,
    pub image: Option<Bytes>,
    pub num_faces: usize,
    pub oracle_text: Option<String>,
}

pub trait GetCardDataInfo {
    fn card_data<'a>(&'a self) -> &'a CardData;

    fn image(&self) -> Option<Bytes> {
        self.card_data().image.as_ref().map(|a| a.clone())
    }

    fn uri(&self) -> Option<String> {
        self.card_data().best_uri()
    }

    fn id(&self) -> &str {
        self.card_data().id.as_str()
    }

    fn name(&self) -> &str {
        self.card_data().name.as_str()
    }

    fn cmc(&self) -> Option<f64> {
        self.card_data().cmc
    }

    fn oracle_text(&self) -> Option<&str> {
        self.card_data().oracle_text.as_ref().map(|a| a.as_str())
    }
}

impl Display for CardData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CardData {{ id: {}, name: {}, cmc: {:?}, num_faces: {} }}",
            self.id, self.name, self.cmc, self.num_faces
        )
    }
}

impl CardData {
    pub fn best_uri(&self) -> Option<String> {
        self.large_uri
            .as_ref()
            .or(self.normal_uri.as_ref())
            .or(self.small_uri.as_ref())
            .map(|a| a.clone())
    }

    pub fn has_image(&self) -> bool {
        self.image.is_some() || self.best_uri().is_some()
    }
}
