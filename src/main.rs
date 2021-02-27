use nsmwe::app::App;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default())
        .expect("Failed to initialize log4rs");
    let app = App::new(800, 600, "NSMWE v0.1.0");
    app.run();
}
