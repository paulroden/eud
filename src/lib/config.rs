use std::borrow::Cow;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use standard_styled::{Colorize, Style};


pub struct Config {
    emacs_exec: String,
    emacs_client_exec: String,
    default_socket: String,
    server_socket_dir: PathBuf,   // c.f. `server-socket-dir' in emacs
    editor: String,
    style: Style,
}

impl Default for Config {
    fn default() -> Self {
        let emacs_exec = env::var("EMACS_EXEC")
            .unwrap_or("emacs".into());
        let emacs_client_exec = env::var("EMACS_CLIENT_EXEC")
            .unwrap_or("emacsclient".into());
        let server_socket_dir = create_server_socket_dir(
            "~/.emacs.d/sockets/"  // CAUTION: hardcoded (but intentionally)
        ).expect("Could not create socket directory at `~/.emacs.d/sockets/` .");

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

        Self {
            emacs_exec,
            emacs_client_exec,
            default_socket: "server".to_string(),
            server_socket_dir,
            editor: "nano".to_string(),
            style: default_style,
        }
    }
}

impl Config {
    pub fn emacs_exec(&self) -> &String {
        &self.emacs_exec
    }

    pub fn emacs_client_exec(&self) -> &String {
        &self.emacs_client_exec
    }

    pub fn alternative_editor(&self) -> &String {
        &self.editor
    }

    pub fn default_socket_name(&self) -> &String {
        &self.default_socket
    }
    pub fn new(
        emacs_exec: String,
        emacs_client_exec: String,
        default_socket: String,
        server_socket_dir: PathBuf,
        editor: String,
        style: Style,
    ) -> Self {
        Self {
            emacs_exec,
            emacs_client_exec,
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


fn expand_tilde_as_home<P: AsRef<Path>>(path: &P) -> Cow<Path> {
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


fn create_server_socket_dir<P>(
    dir_path: P
) -> std::io::Result<PathBuf>
where
  P: AsRef<Path> + AsRef<std::ffi::OsStr>
{
    // NB: we are harmonising this with the following in Emacs' init:
    // `(when (executable-find "eud")
    //     (setq server-socket-dir
    //       (shell-command-to-string "eud server-socket-dir-path")))`
    let server_socket_dir = std::fs::canonicalize(
        expand_tilde_as_home(
            &PathBuf::from(&dir_path)
        )
    )?;
    // create `sockets` directory
    std::fs::create_dir_all(&server_socket_dir)?;
    // .. and ensure permissions are appropriate (rwx------)
    let mut perms = fs::metadata(&server_socket_dir)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(&server_socket_dir, perms)?;

    Ok(server_socket_dir)
}
