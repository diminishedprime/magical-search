use gdk_pixbuf::{prelude::PixbufLoaderExt, PixbufLoader};
use gtk::{prelude::{ContainerExt, GridExt, WidgetExt}, Grid, Image};

use crate::{AppError, GridSearchResult};

pub fn update_grid(grid: &Grid, grid_data: Vec<GridSearchResult>, cols: usize, default_image: &Vec<u8>) -> Result<(), AppError> {
    grid.foreach(|child| grid.remove(child));

    let rows = grid_data.len() / cols;

    // Populate the grid with images and names
    for row in 0..rows {
        for col in 0..cols {
            let idx = row * cols + col;
            if idx >= grid_data.len() {
                break;
            }

            let data = &grid_data[idx];

            let loader = PixbufLoader::new();
            if let Some(ref image_data_small) = data.image_data_small {
                loader.write(&image_data_small).map_err(|_| AppError::WriteError)?;
            } else {
                loader.write(&default_image)?;
            }
            loader.close()?;

            let pixbuf = loader.pixbuf().ok_or(AppError::PixbufError)?;
            let image = Image::from_pixbuf(Some(&pixbuf));

            // Add the image and label to the grid
            grid.attach(&image, row.try_into().unwrap(), col.try_into().unwrap(), 1, 1);
        }
    }
    grid.show_all();
    Ok(())
}
