mod card;
mod card_detail;
mod cards;
mod db;

use async_std::path::PathBuf;
use card::{Card, LoadingCard};
use cards::Cards;
use iced::{
    widget::{column, container, text, TextInput},
    Application, Command, Length, Settings, Theme,
};
use thiserror::Error;

use crate::card_detail::CardDetail;

pub static LIMIT: usize = 9;

enum MagicalSearch {
    Loading,
    Loaded { state: AppState },
}

#[derive(Debug, Clone)]
struct AppState {
    search: String,
    current_cards: Cards,
    selected_card_detail: Option<CardDetail>,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Failed to connect to local database")]
    SQLConnectionError(#[from] tokio_rusqlite::Error),
    #[error("Fail to query successfully")]
    SQLQuery(#[from] rusqlite::Error),
}

#[derive(Debug, Clone)]
enum MessageError {
    SQLConnection,
    SQLQuery,
}

#[derive(Debug, Clone)]
enum Message {
    CardClicked { card_id: String },
    SearchInputChanged(String),
    CardsLoaded(Result<Cards, MessageError>),
    CardLoaded(Result<Card, MessageError>),
    CardDetailLoaded(Result<CardDetail, MessageError>),
}

impl MagicalSearch {
    fn db_path() -> PathBuf {
        PathBuf::from("target").join("cards.sqlite")
    }
}

impl Application for MagicalSearch {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            MagicalSearch::Loading,
            Command::perform(Cards::fetch_cards(), Message::CardsLoaded),
        )
    }

    fn title(&self) -> String {
        match self {
            MagicalSearch::Loading => "Loading...".to_string(),
            MagicalSearch::Loaded { state: _ } => "Magical Search".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::CardsLoaded(result) => {
                let mut commands = Vec::new();
                match result {
                    Ok(cards) => match self {
                        MagicalSearch::Loaded { state } => {
                            for card in &cards.0 {
                                if let Card::LoadingImage(LoadingCard { id, name, .. }) = card {
                                    commands.push(Command::perform(
                                        Card::get_card(id.to_string(), name.to_string()),
                                        Message::CardLoaded,
                                    ));
                                }
                            }
                            state.current_cards = cards;
                        }
                        MagicalSearch::Loading => {
                            for card in &cards.0 {
                                if let Card::LoadingImage(LoadingCard { id, name, .. }) = card {
                                    commands.push(Command::perform(
                                        Card::get_card(id.to_string(), name.to_string()),
                                        Message::CardLoaded,
                                    ));
                                }
                            }
                            *self = MagicalSearch::Loaded {
                                state: AppState {
                                    current_cards: cards,
                                    search: "".to_string(),
                                    selected_card_detail: None,
                                },
                            };
                        }
                    },
                    Err(e) => {
                        panic!("Error loading initial state: {:?}", e);
                    }
                }
                Command::batch(commands)
            }
            Message::SearchInputChanged(ref input) => {
                match self {
                    MagicalSearch::Loaded { state } => {
                        state.search = input.to_string();
                    }
                    _ => (),
                };
                Command::perform(
                    Cards::fetch_cards_with_search(input.to_string()),
                    Message::CardsLoaded,
                )
            }
            Message::CardLoaded(card) => {
                match card {
                    Ok(card) => match self {
                        MagicalSearch::Loading => panic!("I don't think this should ever happen"),
                        MagicalSearch::Loaded { state } => {
                            if let Some(current_card_idx) = state
                                .current_cards
                                .0
                                .iter()
                                .position(|c| c.id() == card.id())
                            {
                                state.current_cards.0[current_card_idx] = card;
                            }
                        }
                    },
                    Err(e) => {
                        todo!("I'm not sure when this would happen, yet: {:?}", e)
                    }
                }
                Command::none()
            }
            Message::CardClicked { card_id } => match self {
                MagicalSearch::Loaded { state } => {
                    if let Some(_) = &state.selected_card_detail {
                        state.selected_card_detail = None;
                        Command::none()
                    } else if let Some(card) =
                        state.current_cards.0.iter().find(|c| c.id() == card_id)
                    {
                        Command::perform(
                            CardDetail::load_card_detail(card.clone()),
                            Message::CardDetailLoaded,
                        )
                    } else {
                        Command::none()
                    }
                }
                _ => Command::none(),
            },
            Message::CardDetailLoaded(card_detail) => {
                match card_detail {
                    Ok(card_detail) => match self {
                        MagicalSearch::Loaded { state } => {
                            state.selected_card_detail = Some(card_detail);
                        }
                        _ => (),
                    },
                    _ => (),
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let content = match self {
            MagicalSearch::Loading => {
                column![text("Loading inital view.").size(40),].width(Length::Shrink)
            }
            MagicalSearch::Loaded { state } => {
                if let Some(selected_card) = &state.selected_card_detail {
                    column![selected_card.view()]
                } else {
                    let cards = state.current_cards.view();
                    let text_input = TextInput::new("Search", &state.search)
                        .on_input(|input| Message::SearchInputChanged(input));
                    column![text_input, cards,]
                }
            }
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

pub fn main() -> iced::Result {
    MagicalSearch::run(Settings::default())
}
