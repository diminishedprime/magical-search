use std::fmt::{self, Display, Formatter};

use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum ImageSize {
    Small,
    Medium,
    Large,
}

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
    pub small: ImageInfo,
    pub normal: ImageInfo,
    pub large: ImageInfo,
    pub num_faces: usize,
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
    pub fn best_uri(&self) -> Option<(String, ImageSize)> {
        self.large
            .uri
            .as_ref()
            .map(|uri| (uri.clone(), ImageSize::Large))
            .or(self
                .normal
                .uri
                .as_ref()
                .map(|uri| (uri.clone(), ImageSize::Medium)))
            .or(self
                .small
                .uri
                .as_ref()
                .map(|uri| (uri.clone(), ImageSize::Small)))
    }
    pub fn best_image(&self) -> Option<(Bytes, ImageSize)> {
        self.large
            .image
            .clone()
            .map(|image| (image, ImageSize::Large))
            .or(self
                .normal
                .image
                .clone()
                .map(|image| (image, ImageSize::Medium)))
            .or(self
                .small
                .image
                .clone()
                .map(|image| (image, ImageSize::Small)))
    }
}
