use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::process::{Child, Command};
use crate::config::Config;
use crate::daemons;


#[derive(Clone, Debug)]
pub struct ClientProcess {
    daemon_socket: PathBuf,
    visit_file: PathBuf,
    alternate_editor: Option<String>,
    create_new_frame: bool,
}

impl ClientProcess {
    fn with_daemon(
        socket_name: impl Into<PathBuf>,
        visit_file: impl Into<PathBuf>
    ) -> Self {
        Self {
            daemon_socket: socket_name.into(),
            visit_file: visit_file.into(),
            alternate_editor: None,
            create_new_frame: true,
         }
    }

    fn spawn(&self) -> Result<Child, std::io::Error> {
        Command::new("emacsclient")
            .arg(
                match &self.create_new_frame {
                    true  => format!("--create-frame"),
                    false => format!("--reuse-frame"),
                }
            )
            .arg(
                format!("--socket-name={}", &self.daemon_socket.display())
            )
            .arg(
                format!(
                    "--alternate-editor={}",
                    &self.alternate_editor.clone().unwrap_or("nano".into())
                )
            )
            .arg(
                format!("{}",
                  fs::canonicalize(&self.visit_file)?.display()
                )
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}


pub fn connect(
    daemon_name: &str,
    file: impl Into<PathBuf>,
    config: &Config
) -> std::io::Result<Child> {
    match daemons::get_all().iter().find(|&p| p.socket_name == daemon_name) {
        Some(daemon) => {
            let socket = daemon.socket_file(config)?;
            let file_path = file.into();
            match file_path.exists() {
                true => ClientProcess::with_daemon(socket, file_path).spawn(),
                false => Err(
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!(
                            "File path {} does not exist.",
                            file_path.display()
                        )
                    )
                ),
            }
        },
        None => Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                {
                    let extant_daemons = daemons::get_all();
                    match extant_daemons.len() {
                        0 => format!("No Emacs daemons are currently running.\n"),
                        _ => format!(
                            "Emacs daemon named `{}` does not exist.\nActive daemons are:\n{}\n",
                            daemon_name,
                            daemons::get_all().iter()
                            .map(|d| d.show(&config))
                            .collect::<Vec<String>>()
                            .join("\n"),
                        )
                    }
                }
            )
        ),
    }
}
