use std::fs;
use std::ops::Deref;
use std::process::{Child, Command};
use std::path::{Path, PathBuf};
use std::env;
use sysinfo::{Pid, Process, ProcessExt, Uid, System, SystemExt};

fn main() {

    let config = Config {
        default_socket: "server",
        tmp_dir: "/tmp/",
    };
    
    list_daemons(&config);
}


fn get_daemons() -> Vec<ProcessInfo> {
    System::new_all().processes().iter()
        .filter(|(_, p)| p.name().to_lowercase().starts_with("emacs"))
        .filter(|(_, p)| match p.cmd().get(1) {
            None => false,
            Some(args) => args.contains("daemon"),
        })
        .map(|(_, p)| ProcessInfo::from_sys_process(p) )
        .collect()
}

fn list_daemons(config: &Config) {
    let daemons = get_daemons();
    
    for daemon in &daemons {
        println!("{}", daemon.show(&config));
    }
}


#[derive(Debug)]
struct Config {
    default_socket: &'static str,
    tmp_dir: &'static str,
}


#[derive(Debug)]
struct Socket {
    file: PathBuf,
}



#[derive(Clone, Debug)]
struct ProcessInfo {
    pid: Pid,
    user_id: Option<Uid>,
    name: String,
    executable: PathBuf,
    command: Vec<String>,
    cwd: PathBuf,
}

impl ProcessInfo {
    fn from_sys_process(p: &Process) -> Self {
        Self {
            pid: p.pid(),
            user_id: p.user_id().cloned(),
            name: p.name().into(),
            executable: p.exe().into(),
            command: p.cmd().to_vec(),
            cwd: p.cwd().into(),
        }
    }

    fn pid(&self) -> Pid {
        self.pid
    }

    fn socket_name(&self) -> Option<String> {
        let name = self.command.get(1)?
            .split_once('=')?
            .1
            .split('\n')
            .last()
            .to_owned()?
            .to_owned();
        Some(name)
    }

    fn socket_file(&self, config: &Config) -> Result<PathBuf, ()> {
        match self.socket_name() {
            Some(name) => {
                match &self.user_id {
                    Some(uid) => {
                        let socket_path = PathBuf::from(config.tmp_dir)
                            .join(format!("emacs{}", uid.deref() ))
                            .join(name);
                        match socket_path.exists() {
                            true => Ok(socket_path),
                            false => {
                                eprintln!("socket file at {:?} does not actually exist, wtf!", socket_path);
                                Err(())
                            }
                        }
                    },
                    None => {
                        eprintln!("No user ID present for ....");
                        Err(())
                    },
                }
            },
            None => {
                eprintln!("no socket!!??");
                Err(())
            }
        }
    }

    fn show(&self, config: &Config) -> String {
        format!(
            "{:<10} [{}, {}]",
            self.socket_name().expect("daemon process with no socket"),
            format!("Pid: {:>8}", format!("{}", self.pid())),
            format!("Socket: {:<24} ",
                self.socket_file(config)
                .expect("problem with socket file...")
                .to_str()
                .expect("path has invalid chars")
            ),
        )
    }
}


/// should return a type which captures either: Child process for a newly-spawned Emacs daemon, or a Process capturing the 
fn launch_emacs_daemon(name: Option<&str>, config: &Config) -> std::io::Result<Child> {
    let daemon_name = match name {
        Some(name) => name,
        None => &config.default_socket,
    };
    Command::new("emacs")
        .arg(format!("--daemon={}", daemon_name))
        .spawn()
}
