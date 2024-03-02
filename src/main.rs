mod card;
mod card_detail;
pub(crate) mod cards;
mod database;
mod db;
mod search;
mod to_sql;
mod types;

use std::iter;

use card::Card;
use cards::Cards;
use iced::{
    widget::{
        column,
        container::{visible_bounds, Id},
        scrollable::Viewport,
        text, Container, TextInput,
    },
    Alignment, Application, Command, Length, Rectangle, Settings, Theme,
};
use search::Search;
use thiserror::Error;

use crate::card_detail::CardDetail;

static INITIAL_SEARCH: &str = "";

use once_cell::sync::Lazy;

static SCROLLABLE_CONTAINER: Lazy<Id> = Lazy::new(|| Id::new("load more content"));

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
    ScrollableVisibleBounds(Option<Rectangle>),
}

impl Application for MagicalSearch {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            MagicalSearch::Loading,
            Cards::initial_rows_for(Search::default()),
        )
    }

    fn title(&self) -> String {
        match self {
            MagicalSearch::Loading => "Loading...".to_string(),
            MagicalSearch::Loaded { state: _ } => "Magical Search".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match self {
            MagicalSearch::Loading => match message {
                Message::LoadRow(ids) => {
                    let ids = ids.expect("I need to figure out better error handling here.");
                    let new_cards = ids.iter().map(|id| Card::loading(id.to_string()));
                    let mut commands: Vec<_> =
                        new_cards.clone().map(|card| card.load_action()).collect();
                    let command = visible_bounds(SCROLLABLE_CONTAINER.clone())
                        .map(Message::ScrollableVisibleBounds);
                    commands.extend(iter::once(command));
                    *self = MagicalSearch::Loaded {
                        state: AppState {
                            search: INITIAL_SEARCH.to_string(),
                            current_cards: Cards::new(new_cards.collect()),
                            selected_card_detail: None,
                        },
                    };
                    Command::batch(commands)
                }
                _ => Command::none(),
            },
            MagicalSearch::Loaded { state } => match message {
                Message::SearchInputChanged(ref input) => {
                    let search = search::search(input);
                    state.search = input.to_string();
                    state.current_cards.clear();
                    if let Ok(search) = search {
                        Cards::initial_rows_for(search)
                    } else if input == "" {
                        Cards::initial_rows_for(Search::default())
                    } else {
                        Command::none()
                    }
                }
                Message::CardLoaded(card) => {
                    match card {
                        Ok(card) => {
                            if let Some(current_card_idx) = state
                                .current_cards
                                .contents
                                .iter()
                                .position(|c| c.id() == card.id())
                            {
                                state.current_cards.contents[current_card_idx] = card;
                            };
                        }
                        Err(e) => {
                            todo!("I'm not sure when this would happen, yet: {:?}", e)
                        }
                    }
                    Command::none()
                }
                Message::CardClicked { card_id } => {
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
                Message::CardDetailLoaded(card_detail) => {
                    match card_detail {
                        Ok(card_detail) => state.selected_card_detail = Some(card_detail),
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
                        if let MagicalSearch::Loaded { state } = self {
                            let cursor = state.current_cards.cursor.clone();
                            let search = search::search(&state.search).unwrap_or(Search::default());
                            return Command::perform(
                                Cards::next_row(cursor, search),
                                Message::LoadRow,
                            );
                        }
                    }
                    Command::none()
                }
                Message::LoadRow(ids) => {
                    let ids = ids.expect("I need to figure out better error handling here.");
                    // Stop if there are no more cards to load.
                    if ids.len() == 0 {
                        return Command::none();
                    }
                    let new_cards = ids.iter().map(|id| Card::loading(id.to_string()));
                    let mut commands: Vec<_> =
                        new_cards.clone().map(|card| card.load_action()).collect();
                    state.current_cards.extend_cards(new_cards);
                    println!("Adding command to check the bounds.");
                    let command = visible_bounds(SCROLLABLE_CONTAINER.clone())
                        .map(Message::ScrollableVisibleBounds);
                    commands.extend(iter::once(command));
                    Command::batch(commands)
                }
                Message::ScrollableVisibleBounds(rect) => {
                    if rect.is_some() {
                        println!("Loading next row since the end is visible.");
                        let cursor = state.current_cards.cursor.clone();
                        let search = search::search(&state.search).unwrap_or(Search::default());
                        Command::perform(Cards::next_row(cursor, search), Message::LoadRow)
                    } else {
                        Command::none()
                    }
                }
            },
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
                    let visible_check =
                        Container::new(text("End of the line")).id(SCROLLABLE_CONTAINER.clone());
                    let cards = state.current_cards.view();
                    let text_input = TextInput::new("Search", &state.search)
                        .on_input(|input| Message::SearchInputChanged(input));
                    column![text_input, cards, visible_check]
                        .align_items(Alignment::Center)
                        .padding(SPACING_SMALL)
                }
            }
        };

        Container::new(
            iced::widget::scrollable(content).on_scroll(|viewport| Message::Scrolled(viewport)),
        )
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
