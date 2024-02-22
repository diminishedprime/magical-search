mod card_grid;

use gdk_pixbuf::glib;
use gtk::Entry;
use gtk::{prelude::*, Application, ApplicationWindow, Grid};
use reqwest::blocking::Client;
use rusqlite::Connection;
use rusqlite::Error as RusqliteError;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to write image data to loader")]
    WriteError,
    #[error("Failed to close loader")]
    CloseError,
    #[error("Failed to get Pixbuf from loader")]
    PixbufError,
    #[error("Failed to prepare SQL statement")]
    SQLPrepareError,
    #[error("Failed to connect to local database")]
    SQLConnectionError,
    #[error("Failed to execute SQL query")]
    SQLQueryError(#[from] RusqliteError),
    #[error("Generic glib Error")]
    GlibError(#[from] glib::Error),
    #[error("Request Error")]
    RequestError(#[from] reqwest::Error),
}

#[derive(Debug)]
pub(crate) struct GridSearchResult {
    pub(crate) _name: String,
    pub(crate) _cmc: f64,
    pub(crate) image_data_small: Option<Vec<u8>>,
}

static GET_DEFAULT_IMAGE_URI_SQL: &str = r#"
SELECT small
FROM card_image_uris
WHERE card_id = :card_id
LIMIT 1;
"#;

static GET_GRID_DATA_SQL: &str = r#"
SELECT c.name, c.cmc, ciu.small AS small_image_uri
FROM cards c
JOIN card_image_uris ciu ON c.id = ciu.card_id
LIMIT :limit;
"#;

static GET_GRID_DATA_WITH_SEARCH: &str = r#"
SELECT c.name, c.cmc, ciu.small AS small_image_uri
FROM cards c
JOIN card_image_uris ciu ON c.id = ciu.card_id
WHERE c.name LIKE :search
LIMIT :limit;
"#;

// TODO - I could probably pick a better default image here.
fn get_default_image(connection: &Connection) -> Result<Vec<u8>, AppError> {
    let saproling_token_id = "006c118e-b5c7-4726-acee-59132f23e4fc";
    let image_uri = connection
        .prepare(GET_DEFAULT_IMAGE_URI_SQL)?
        .query_map(&[(":card_id", &saproling_token_id)], |row| Ok(row.get(0)?))?
        .collect::<Result<Vec<String>, _>>()?;
    let image_uri = image_uri
        .get(0)
        .ok_or(AppError::SQLQueryError(RusqliteError::QueryReturnedNoRows))?;
    download_image(image_uri)
}

fn download_image(uri: &str) -> Result<Vec<u8>, AppError> {
    let client = Client::new();
    let response = client.get(uri).send()?;
    let body = response.bytes()?;
    Ok(body.to_vec())
}

fn grid_search_results(
    connection: &Connection,
    filter_text: &str,
    max_results: usize,
) -> Result<Vec<GridSearchResult>, AppError> {
    // TODO - eventually the filter_text should support what scryfall supports.
    Ok(if filter_text.is_empty() {
        let grid_data = connection
            .prepare(GET_GRID_DATA_SQL)?
            .query_map(&[(":limit", &max_results.to_string())], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<Result<Vec<(String, f64, Option<String>)>, _>>()?;
        grid_data
            .iter()
            .map(|(name, cmc, image_uri)| {
                let image_data = if let Some(uri) = image_uri {
                    Some(download_image(uri)?)
                } else {
                    None
                };
                Ok(GridSearchResult {
                    _name: name.to_string(),
                    _cmc: *cmc,
                    image_data_small: image_data,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?
    } else {
        let grid_data = connection
        .prepare(GET_GRID_DATA_WITH_SEARCH)?.
        query_map(
            &[(":search", &format!("%{}%", filter_text)), (":limit", &max_results.to_string())], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<(String, f64, Option<String>)>, _>>()?;
        grid_data
            .iter()
            .map(|(name, cmc, image_uri)| {
                let image_data = if let Some(uri) = image_uri {
                    Some(download_image(uri)?)
                } else {
                    None
                };
                Ok(GridSearchResult {
                    _name: name.to_string(),
                    _cmc: *cmc,
                    image_data_small: image_data,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?
    })
}

fn main() -> Result<(), AppError> {
    let application = Application::builder()
        .application_id("io.mjh.magical_search")
        .build();
    let target_dir = PathBuf::from("target");
    let cards_db_file_path = target_dir.join("cards.sqlite");
    let conn = Arc::new(
        rusqlite::Connection::open(cards_db_file_path).map_err(|_| AppError::SQLConnectionError)?,
    );
    let default_image = get_default_image(&conn)?;
    application.connect_activate(move |app| {
        let initial_grid_data =
            grid_search_results(&conn.clone(), "", 16).expect("Failed to get initial grid data");

        let grid = Grid::builder()
            .column_homogeneous(true)
            .row_homogeneous(true)
            .column_spacing(4)
            .row_spacing(4)
            .build();

        card_grid::update_grid(&grid, initial_grid_data, 4, &default_image)
            .expect("Failed to update the gui grid");

        let entry = Entry::builder().placeholder_text("Filter...").build();

        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .build();
        vbox.pack_start(&entry, false, false, 8);
        vbox.pack_start(&grid, false, false, 8);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Magical Search")
            .default_height(800)
            .default_width(600)
            .build();

        window.set_child(Some(&vbox));

        window.show_all();

        let cloned_grid = grid.clone();
        let entry_conn = conn.clone();
        let default_image = default_image.clone();
        entry.connect_changed(move |entry| {
            let updated_grid_data = grid_search_results(&entry_conn, &entry.text(), 16)
                .expect("Failed to get initial grid data");

            card_grid::update_grid(&cloned_grid, updated_grid_data, 4, &default_image)
                .expect("Failed to update gui grid");
        });
    });

    application.run();
    Ok(())
}
