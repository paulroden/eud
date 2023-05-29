use std::borrow::Cow;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use standard_styled::{Colorize, Style};


pub struct Config {
    default_socket: String,
    server_socket_dir: PathBuf,   // c.f. `server-socket-dir' in emacs
    editor: String,
    style: Style,
}

impl Config {
    pub fn alternative_editor(&self) -> &String {
        &self.editor
    }
    pub fn default() -> std::io::Result<Self> {
        // NB: we are harmonising this with the following in Emacs' init:
        // ``
        let server_socket_dir = std::fs::canonicalize(
            expand_tilde_as_home(
                &PathBuf::from("~/.emacs.d/sockets/")
            )
        )?;
        // create `sockets` directory and ensure permissions are appropriate (rwx------)
        std::fs::create_dir_all(&server_socket_dir)?;
        let mut perms = fs::metadata(&server_socket_dir)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&server_socket_dir, perms)?;

        let default_style = Style {
            spinner: vec![
                "(●     )",
                "( ●    )",
                "(  ●   )",
                "(   ●  )",
                "(    ● )",
                "(     ●)",
            ],
            stdout_style: Box::new(|s: &str| s.blue() ),
            stderr_style: Box::new(|s: &str| s.yellow() ),
            message_style: Box::new(|s: &str| s.bold().truecolor(127, 90, 182) ),
            end_message: Some(" Launched Emacs daemon  🚀 ".to_string()),
        };
        
        Ok(Self {
            default_socket: "server".to_string(),
            server_socket_dir,
            editor: "nano".to_string(),
            style: default_style,
        })
    }
    pub fn default_socket_name(&self) -> &String {
        &self.default_socket
    }
    pub fn new(
        default_socket: String,
        server_socket_dir: PathBuf,
        editor: String,
        style: Style,
    ) -> Self {
        Self {
            default_socket,
            server_socket_dir,
            editor,
            style,
        }
    }
    pub fn server_socket_dir(&self) -> &PathBuf {
        &self.server_socket_dir
    }
    pub fn style(&self) -> &Style {
        &self.style
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

