mod filtro;
mod ui;

use gtk4::prelude::*;
use gtk4::{Application, glib};

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("dev.duck.launcher")
        .build();
    app.connect_activate(ui::build_ui);
    app.run()
}
