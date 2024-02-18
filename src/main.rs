mod card_grid;

use gdk_pixbuf::glib;
use gtk::Entry;
use gtk::{prelude::*, Application, ApplicationWindow, Grid};
use rusqlite::Connection;
use rusqlite::Error as RusqliteError;
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
}

#[derive(Debug)]
pub(crate) struct GridSearchResult {
    pub(crate) _name: String,
    pub(crate) _cmc: f64,
    pub(crate) image_data_small: Option<Vec<u8>>,
}

// TODO - I could probably pick a better default image here.
fn get_default_image(
    connection: &Connection
) -> Result<Vec<u8>, AppError> {
    let saproling_token_id = "006c118e-b5c7-4726-acee-59132f23e4fc";
    let result = connection
        .prepare(&format!("SELECT image_data_small FROM cards WHERE id='{id}' LIMIT 1", id=saproling_token_id))?.query_map([], |row| {
            Ok(row.get(0)?)
        })?.collect::<Result<Vec<_>, _>>()?;
    result.get(0).map(|x:&Vec<u8>| x.clone()).ok_or(AppError::SQLQueryError(RusqliteError::QueryReturnedNoRows))
}

fn grid_search_results(
    connection: &Connection,
    filter_text: &str,
    max_results: usize,
) -> Result<Vec<GridSearchResult>, AppError> {
    // TODO - eventually the filter_text should support what scryfall supports.
    Ok(if filter_text.is_empty() {
        connection
        .prepare(&format!("SELECT DISTINCT name, cmc, image_data_small FROM cards WHERE lang='en' LIMIT {max_results}", max_results=max_results))?.query_map([], |row| {
            Ok(GridSearchResult {
                _name: row.get(0)?,
                _cmc: row.get(1)?,
                image_data_small: row.get(2)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?
    } else {
        connection
        .prepare(&format!("SELECT DISTINCT name, cmc, image_data_small FROM cards WHERE lang='en' and name LIKE ? LIMIT {max_results}", max_results=max_results))?.
        query_map([&format!("%{}%", filter_text)], |row| {
            Ok(GridSearchResult {
                _name: row.get(0)?,
                _cmc: row.get(1)?,
                image_data_small: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    })
}

fn main() -> Result<(), AppError> {
    let application = Application::builder()
        .application_id("io.mjh.magical_search")
        .build();
    let conn = Arc::new(
        rusqlite::Connection::open("cards.sqlite").map_err(|_| AppError::SQLConnectionError)?,
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

        card_grid::update_grid(&grid, initial_grid_data, 4, &default_image).expect("Failed to update the gui grid");

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
