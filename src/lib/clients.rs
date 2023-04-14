use std::process::{Child, Command};
use std::path::PathBuf;

use crate::config::Config;
use crate::daemons::{get_daemons, list_daemons};

#[derive(Clone, Debug)]
pub struct ClientProcess {
    daemon_socket: PathBuf,
    alternate_editor: Option<String>,
    create_new_frame: bool,
}

impl ClientProcess {
    fn with_daemon(socket_name: impl Into<PathBuf>) -> Self {
        Self {
            daemon_socket: socket_name.into(),
            alternate_editor: None,
            create_new_frame: true,
         }
    }

    fn spawn(&self) -> Result<Child, std::io::Error> {
        Command::new("emacsclient")
            .arg(
                format!("--socket-name={}", self.daemon_socket.display())
            )
            .arg(
                match &self.create_new_frame {
                    true  => format!("--create-frame"),
                    false => format!("--reuse-frame"),
                }
            )
            .arg(
                match &self.alternate_editor {
                    Some(editor) => format!("--alternate_editor={}", editor),
                    None => "".to_string(),
                }
            )
           .spawn()
    }
}


pub fn launch_client(daemon_name: &str, config: &Config) -> Result<Child, std::io::Error> {
    match get_daemons().iter().find(|&p| p.socket_name == daemon_name) {
        Some(daemon) => {
            let socket = daemon.socket_file(config)?;
            ClientProcess::with_daemon(socket).spawn()
        } 
        None => Err(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Emacs daemon named {:?} does not exist.\nActive daemons are: {:?}",
                    daemon_name,
                    list_daemons(&config).unwrap(),
            ))),
    }
}
