use eud::config::Config;
use eud::cli::cli;

fn main() {
    match cli(&Config::default()) {
        Ok(_) => (),
        Err(e) => eprint!("{}", e),
    }
}
