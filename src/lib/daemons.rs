use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use sysinfo::{Pid, Process, ProcessExt, Signal, System, SystemExt, Uid};

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct DaemonProcess {
    pub pid: Pid,
    pub user_id: Option<Uid>,
    pub socket_name: String,
}

impl DaemonProcess {
    pub(crate) fn from_sys_process(p: &Process) -> Option<Self> {
        // The socket name needs to be derived from the command arguments
        // passed to emacs. These will be of the form:
        // --bg-daemon=\xxx,y\012/name//or/socket/path
        // The result of `p.cmd()` is therefore parsed to extract the
        // "/name//or/socket/path" portion into a `Path`, to extract the
        // socket filename
        let socket_name = Path::new(p.cmd()
            .get(1)?
            .split_once('=')?
            .1
            .split('\n')
            .last()?
        ).file_name()?.to_str();

        Some(Self {
            pid: p.pid(),
            user_id: p.user_id().cloned(),
            socket_name: socket_name?.to_owned(),
        })
    }

    pub(crate) fn kill(&self) -> Result<Pid, std::io::Error> {
        let system = System::new_all();
        let pid = self.pid;
        match system.process(pid) {
            // Process should be killed with TERM signal (15),
            // this is consistent with `kill PID` on MacOS and allows
            // the Emacs daemon process to clear up its socket file.
            Some(process) => match process.kill_with(Signal::Term) {
                Some(true) => Ok(pid),
                Some(false) => Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Error trying to send kill signal to Emacs daemon '{}' with Pid {}.",
                            self.socket_name,
                            pid
                        )
                    )
                ),
                None => Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Signal::Term does not exist on this system."
                    )
                ),
            },
            None => Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Error trying to send kill signal to Emacs daemon. No process found with with Pid {}.", pid)
                )
            )
        }
    }

    pub(crate) fn show(&self, config: &Config) -> String {
        format!(
            "{:<14} [{}, {}]",
            self.socket_name,
            format!("Pid: {:>8}", format!("{}", self.pid)),
            format!(
                "Socket: {:<30} ",
                self.socket_file(config)
                    .expect("problem with socket file...")  // this exception can be reached if an emacs daemon process continues but the socket has been closed... why/how/handle?
                    .to_str()
                    .expect("path has invalid chars")
            ),
        )
    }

    pub(crate) fn socket_file(&self, config: &Config) -> Result<PathBuf, std::io::Error> {
        match &self.user_id {
            Some(uid) => {
                let socket_path = PathBuf::from(&config.server_socket_dir())
                    .join(format!("emacs{}", uid.deref()))
                    .join(self.socket_name.clone());
                match socket_path.exists() {
                    true => Ok(socket_path),
                    false => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Daemon socket at path {} does not exist.",
                            socket_path.display()
                        ),
                    )),
                }
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Unexpected! No user ID present for Emacs daemon process:\n{:?}",
                    self
                ),
            )),
        }
    }
}

pub(crate) fn get_all() -> Vec<DaemonProcess> {
    System::new_all()
        .processes()
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.name().to_lowercase().starts_with("emacs"))
        .filter(|p| p.cmd().get(1)
            .map_or_else(
                || false,
                |args| args.contains("daemon"))
        )
        .map(|p| DaemonProcess::from_sys_process(p))
        .flatten()
        .collect()
}

pub(crate) fn active_daemons_names() -> Vec<String> {
    get_all().iter().map(|d| d.socket_name.clone()).collect()
}

pub(crate) fn launch_new(
    name: &Option<String>,
    config: &Config
) -> std::io::Result<Child> {
    let daemon_name = match name {
        Some(name) => name,
        None => &config.default_socket,
    };
    Command::new("emacs")
        .arg(format!("--daemon={}", daemon_name))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
// TODO: (above) look into std::process::Command::{current_dir, envs}

pub(crate) fn kill_by_name(name: &str) -> Result<Pid, std::io::Error> {
    get_all()
        .iter()
        .find(|&p| p.socket_name == name)
        .map_or_else(
            || Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("No Emacs daemon found with socket name {name}"),
            )),
            |daemon| daemon.kill(),
        )
}

pub(crate) fn kill_all() -> Vec<Result<Pid, std::io::Error>> {
    get_all().iter().map(|daemon| daemon.kill()).collect()
}
