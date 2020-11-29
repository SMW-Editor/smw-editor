extern crate nsmwe_app;

use nsmwe_app::App;

fn main() {
    let app = App::new(800, 600, "NSMWE v0.1.0");
    app.run();
}