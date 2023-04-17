use eud::config::Config;
use eud::cli::cli;

fn main() {

    let config = Config::new(
        "server",
        "/tmp/",
        "nano",
    );

    match cli(&config) {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    };
}
