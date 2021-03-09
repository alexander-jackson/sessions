fn main() {
    dotenv::dotenv().unwrap();

    let filters = vec![("blackboards", log::LevelFilter::Trace)];
    blackboards::setup_logger_with_filters(filters);

    let rocket = blackboards::build_rocket();
    rocket.launch();
}
