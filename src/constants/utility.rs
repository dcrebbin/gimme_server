pub fn is_development() -> bool {
    std::env::var("ENV").unwrap_or_default() == "development"
}

pub fn log_query(query: &str) {
    if is_development() {
        println!("{:?}", query);
    }
}

pub fn log_error(error: &str) {
    if is_development() {
        eprintln!("{:?}", error);
    }
}
