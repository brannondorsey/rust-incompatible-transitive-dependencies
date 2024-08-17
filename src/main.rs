use a::log as log_a;
use b::log as log_b;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .init()
        .expect("Failed to initialize logger");
    log_a();
    log_b();
}
