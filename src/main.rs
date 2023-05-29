use eud::config::Config;
use eud::cli::cli;

fn main() {

    match Config::default() {
        Ok(config) => {
            match cli(&config) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            }
        },
        Err(e) => eprintln!("Error with default config:\n{e}"),
    }
}


