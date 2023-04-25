use std::borrow::Cow;
use std::path::{Path, PathBuf};


#[derive(Debug)]
pub struct Config {
    default_socket: String,
    server_socket_dir: PathBuf,   // c.f. `server-socket-dir' in emacs
    editor: String,
}

impl Config {
    pub fn new(
        default_socket: String,
        server_socket_dir: PathBuf,
        editor: String,
    ) -> Self {
        Self {
            default_socket,
            server_socket_dir,
            editor,
        }
    }
    pub fn default() -> std::io::Result<Self> {
        // NB: we are harmonising this with the following in Emacs' init:
        // ``
        let server_socket_dir = std::fs::canonicalize(
            expand_tilde_as_home(
                &PathBuf::from("~/.emacs.d/sockets/")
            )
        )?;
        std::fs::create_dir_all(&server_socket_dir)?;
        Ok(Self {
            default_socket: "server".to_string(),
            server_socket_dir,
            editor: "nano".to_string(),
        })
    }
    pub fn default_socket_name(&self) -> &String {
        &self.default_socket
    }
    pub fn server_socket_dir(&self) -> &PathBuf {
        &self.server_socket_dir
    }
    pub fn alternative_editor(&self) -> &String {
        &self.editor
    }
}


fn expand_tilde_as_home<'p, P: AsRef<Path>>(path: &'p P) -> Cow<'p, Path> {
    let path = path.as_ref();

    match path.starts_with("~") {
        true => dirs::home_dir()
            .expect("Error: unable to determine home directory (~) for operating system.")
            // unwrap should be unreachable here since this match arm follows `.starts_with("~")`
            .join(path.strip_prefix("~").unwrap())  
            .into(),
        false => path.into()
    }
}

