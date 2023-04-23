use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub default_socket: String,
    pub server_socket_dir: PathBuf,   // c.f. `server-socket-dir' in emacs
    pub editor: String,
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
        let server_socket_dir = PathBuf::from("~/.emacs.d/sockets/");
        std::fs::create_dir_all(&server_socket_dir)?;
        Ok(Self {
            default_socket: "server".to_string(),
            server_socket_dir,
            editor: "nano".to_string(),
        })
    }
    pub fn server_socket_dir(&self) -> PathBuf {
        self.server_socket_dir.clone()
    }
}


// Socket Name
// socket_name
//   = as passed to cli (i.e. `new NAME`)
//   | per environment $EMACS_SERVER_NAME (?)
//   | config file default (?)
//   | Emacs own default ("server"), cf. `server-name'  https://github.com/emacs-mirror/emacs/blob/4f3dae2b0d5fc43e5e2effa6d36544b6de2a43d8/lisp/server.el#L253

// Socket File Directory
// sockets_dir
//   = as passed to cli (?)
//   | per environment $EMACS_SOCKET_DIR (?)
//   | config file default (?)
//   | Emacs own default ($TMPDIR or /tmp)   i.e. `server-socket-dir' https://github.com/emacs-mirror/emacs/blob/4f3dae2b0d5fc43e5e2effa6d36544b6de2a43d8/lisp/server.el#L283
// also consider making this a common variable read by Emacs' init and eud

// Alternative Text Editor
// alternative_editor
// ...

// others config variables?

