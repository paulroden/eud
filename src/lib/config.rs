

#[derive(Debug)]
pub struct Config {
    pub default_socket: &'static str,
    pub tmp_dir: &'static str,
}

impl Config {
    pub fn new(default_socket: &'static str, tmp_dir: &'static str) -> Self {
        Self {
            default_socket,
            tmp_dir,
        }
    }
}
