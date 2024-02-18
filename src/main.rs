use gdk_pixbuf::{Pixbuf, PixbufLoader};
use gtk::{prelude::*, CellRendererPixbuf, CellRendererText};
use gtk::{Entry, ListStore, TreeView, TreeViewColumn, Window, WindowType};

fn main() {
    // Initialize GTK application
    gtk::init().expect("Failed to initialize GTK.");

    // Create a new window
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Magical Search");
    window.set_default_size(400, 300);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);

    // Create a text entry for filter input
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Filter..."));

    // Create a TreeView to display the data
    let treeview = TreeView::new();

    // Create a ListStore to store the data
    let store = ListStore::new(&[
        String::static_type(),
        f64::static_type(),
        Pixbuf::static_type(),
    ]);
    treeview.set_model(Some(&store));

    // Create the first column for the "name" attribute
    let column1 = TreeViewColumn::new();
    let cell1 = CellRendererText::new();
    gtk::prelude::TreeViewColumnExt::pack_start(&column1, &cell1, true);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column1, &cell1, "text", 0);
    column1.set_title("Name");
    treeview.append_column(&column1);

    // Create the second column for the "cmc" attribute
    let column2 = TreeViewColumn::new();
    let cell2 = CellRendererText::new();
    gtk::prelude::TreeViewColumnExt::pack_start(&column2, &cell2, true);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column2, &cell2, "text", 1);
    column2.set_title("CMC");
    treeview.append_column(&column2);

    // Create the third column for the "image_uri_small" attribute
    let column3 = TreeViewColumn::new();
    let cell3 = CellRendererPixbuf::new();
    gtk::prelude::TreeViewColumnExt::pack_start(&column3, &cell3, true);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column3, &cell3, "pixbuf", 2);
    column3.set_title("Image");
    treeview.append_column(&column3);

    // Populate the ListStore with data from the SQLite database
    update_list_store(&store, "").expect("Failed to populate list store");

    // Handle text entry changes to update the filter
    entry.connect_changed(move |entry| {
        update_list_store(&store, &entry.text())
            .map_err(|e| format!("{:?}", e))
            .expect("Failed to update list store");
        entry.queue_draw()
    });

    vbox.pack_start(&entry, false, false, 0);
    vbox.pack_start(&treeview, false, false, 0);
    window.add(&vbox);

    // Show the window and start the GTK main event loop
    window.show_all();
    gtk::main();
}

fn update_list_store(
    store: &ListStore,
    filter_text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to SQLite database
    let conn = rusqlite::Connection::open("cards.sqlite")?;

    // Construct the SQL query with optional filtering
    let query = if filter_text.is_empty() {
        "SELECT DISTINCT name, cmc, image_data_small FROM cards LIMIT 3"
    } else {
        "SELECT DISTINCT name, cmc, image_data_small FROM cards WHERE name LIKE ? LIMIT 3"
    };

    // Retrieve data from the database
    let mut stmt = conn.prepare(query)?;

    let rows = if filter_text.is_empty() {
        stmt.query_map([], |row| {
            Ok((
                row.get::<usize, String>(0)?,
                row.get::<usize, f64>(1)?,
                row.get::<usize, Vec<u8>>(2)?,
            ))
        })?.collect::<Result<Vec<_>, _>>()
    } else {
        stmt.query_map([&format!("%{}%", filter_text)], |row| {
            Ok((
                row.get::<usize, String>(0)?,
                row.get::<usize, f64>(1)?,
                row.get::<usize, Vec<u8>>(2)?,
            ))
        })?.collect::<Result<Vec<_>, _>>()
    }?;

    // Clear the ListStore
    store.clear();

    // Add data to the ListStore
    for (name, cmc, image_uri) in rows {
        let actual_image =
            process_image(image_uri).expect("I don't know how to do this error handling yet");
        store.insert_with_values(None, &[(0, &name), (1, &cmc), (2, &actual_image)]);
    }

    Ok(())
}

fn process_image(data: Vec<u8>) -> Option<Pixbuf> {
    let loader = PixbufLoader::new();
    loader.write(&data).ok()?;
    loader.close().ok()?;
    loader.pixbuf()
}
