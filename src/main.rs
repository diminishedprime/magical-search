mod card;
mod card_detail;
pub(crate) mod cards;
mod database;
mod db;
mod search;
mod to_sql;
mod types;

use card::Card;
use cards::Cards;
use iced::{
    widget::{
        column, scrollable as make_scrollable, scrollable::Viewport, text, Container, TextInput,
    },
    Alignment, Application, Command, Length, Settings, Theme,
};
use search::Search;
use thiserror::Error;

use crate::card_detail::CardDetail;

static INITIAL_SEARCH: &str = "";
pub static CARDS_PER_ROW: usize = 3;
pub static ROWS: usize = 3;
pub static LIMIT: usize = CARDS_PER_ROW * ROWS;

const SPACING_SMALL: u16 = 2;
// const SPACING_MEDIUM: u16 = SPACING_SMALL * 2;
// const SPACING_LARGE: u16 = SPACING_SMALL * 3;

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

#[derive(Debug, Clone, thiserror::Error)]
enum MessageError {
    #[error("Failed to connect to local database")]
    SQLConnection,
    #[error("Fail to query successfully")]
    SQLQuery,
}

#[derive(Debug, Clone)]
enum Message {
    CardClicked { card_id: String },
    NextFace { card_id: String },
    SearchInputChanged(String),
    CardLoaded(Result<Card, MessageError>),
    CardDetailLoaded(Result<CardDetail, MessageError>),
    Scrolled(Viewport),
    LoadRow(Result<Vec<String>, MessageError>),
}

impl Application for MagicalSearch {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            MagicalSearch::Loading,
            // Command::perform(Cards::next_row(0, Search::and(vec![])), Message::LoadRow),
            // TODO - This is what I'd like to avoid
            Command::batch(vec![
                Command::perform(Cards::next_row(0, Search::and(vec![])), Message::LoadRow),
                Command::perform(Cards::next_row(2, Search::and(vec![])), Message::LoadRow),
                Command::perform(Cards::next_row(5, Search::and(vec![])), Message::LoadRow),
            ]),
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
            Message::SearchInputChanged(ref input) => {
                let search = search::search(input);
                match search::search(input) {
                    Ok(query) => println!("Parsed query: {:?}", query),
                    Err(_) => println!("Error parsing query"),
                }
                match self {
                    MagicalSearch::Loaded { state } => {
                        state.search = input.to_string();
                    }
                    _ => (),
                };
                if let Ok(search) = search {
                    Command::perform(Cards::next_row(0, search), Message::LoadRow)
                } else if input == "" {
                    Command::perform(Cards::next_row(0, Search::and(vec![])), Message::LoadRow)
                } else {
                    Command::none()
                }
            }
            Message::CardLoaded(card) => {
                match card {
                    Ok(card) => match self {
                        MagicalSearch::Loading => panic!("I don't think this should ever happen"),
                        MagicalSearch::Loaded { state } => {
                            println!(
                                "Checking to see if loaded card is in current cards: {:?}",
                                card.id()
                            );
                            if let Some(current_card_idx) = state
                                .current_cards
                                .contents
                                .iter()
                                .position(|c| c.id() == card.id())
                            {
                                println!("Card was found in current cards: {:?}", card.id());
                                state.current_cards.contents[current_card_idx] = card;
                            };
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
                    } else if let Some(card) = state
                        .current_cards
                        .contents
                        .iter()
                        .find(|c| c.id() == card_id)
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
            Message::NextFace { card_id, .. } => {
                if let MagicalSearch::Loaded { state } = self {
                    if let Some(idx) = state
                        .current_cards
                        .contents
                        .iter()
                        .position(|c| c.id() == card_id)
                    {
                        if let Card::ArtSeries(ref mut art_series) =
                            state.current_cards.contents[idx]
                        {
                            return Command::perform(
                                Card::next_card_face(card_id.clone(), art_series.selected_face),
                                Message::CardLoaded,
                            );
                        }
                    }
                }
                Command::none()
            }
            Message::Scrolled(viewport) => {
                if viewport.relative_offset().y >= 1.0 {
                    println!("getting next row.");
                    if let MagicalSearch::Loaded { state } = self {
                        let cursor = state.current_cards.cursor.clone();
                        let search = search::search(&state.search).unwrap_or(Search::and(vec![]));
                        return Command::perform(Cards::next_row(cursor, search), Message::LoadRow);
                    }
                }
                Command::none()
            }
            Message::LoadRow(ids) => {
                let ids = ids.expect("I need to figure out better error handling here.");
                let new_cards = ids.iter().map(|id| Card::loading(id.to_string()));
                let commands: Vec<_> = new_cards.clone().map(|card| card.load_action()).collect();
                match self {
                    MagicalSearch::Loading => {
                        *self = MagicalSearch::Loaded {
                            state: AppState {
                                search: INITIAL_SEARCH.to_string(),
                                current_cards: Cards::new(new_cards.collect()),
                                selected_card_detail: None,
                            },
                        };
                    }
                    MagicalSearch::Loaded { state } => {
                        state.current_cards.extend_cards(new_cards);
                    }
                }
                Command::batch(commands)
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
                        .align_items(Alignment::Center)
                        .padding(SPACING_SMALL)
                }
            }
        };

        Container::new(make_scrollable(content).on_scroll(|viewport| Message::Scrolled(viewport)))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(SPACING_SMALL)
            .center_x()
            .into()
    }
}

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size.height = settings.window.size.height + 200.0;
    MagicalSearch::run(settings)
}
