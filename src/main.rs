use std::fs;
use std::ops::Deref;
use std::process::{Child, Command};
use std::path::PathBuf;
use std::env;
use sysinfo::{Pid, Process, ProcessExt, Uid, Signal, System, SystemExt};

fn main() {

    let config = Config {
        default_socket: "server",
        tmp_dir: "/tmp/",
    };
    
    list_daemons(&config).unwrap();

    println!("{:?}", active_daemons_names());

    // let new_daemon = launch_emacs_daemon(Some("test-daemon"), &config);


    match kill_daemon("test-daemon-2") {
        Ok(_) => println!("Killed it."),
        Err(e) => {
            eprintln!("{}", e);
            list_daemons(&config).unwrap();
        },
    }
    
    println!("supported signals: {:?}", System::SUPPORTED_SIGNALS);
}


fn get_daemons() -> Vec<DaemonProcess> {
    System::new_all().processes().iter()
        .filter(|(_, p)| p.name().to_lowercase().starts_with("emacs"))
        .filter(|(_, p)| match p.cmd().get(1) {
            None => false,
            Some(args) => args.contains("daemon"),
        })
        .map(|(_, p)| DaemonProcess::from_sys_process(p) )
        .flatten()
        .collect()
}


#[derive(Debug)]
struct Config {
    default_socket: &'static str,
    tmp_dir: &'static str,
}


fn list_daemons(config: &Config) -> Result<(), std::io::Error> {
    let daemons = get_daemons();

    println!("Current Emacs daemon instances:");
    
    for daemon in &daemons {
        println!("{}", daemon.show(&config));
    }
    
    Ok(())
}

fn active_daemons_names() -> Vec<String> {
    get_daemons().iter()
        .map(|d| d.socket_name.clone())
        .collect()
}


#[derive(Clone, Debug)]
struct DaemonProcess {
    pid: Pid,
    user_id: Option<Uid>,
    name: String,
    socket_name: String,
    // executable: PathBuf,
    // command: Vec<String>,
    // cwd: PathBuf,
}

impl DaemonProcess {
    fn from_sys_process(p: &Process) -> Option<Self> {
        let socket_name = p.cmd().get(1)?
            .split_once('=')?
            .1
            .split('\n')
            .last();

        match socket_name {
            Some(socket_name) => Some(
                Self {
                    pid: p.pid(),
                    user_id: p.user_id().cloned(),
                    name: p.name().into(),
                    socket_name: socket_name.to_owned(),
                }
            ),
            None => None,
        }
    }

    fn show(&self, config: &Config) -> String {
        format!(
            "{:<14} [{}, {}]",
            self.socket_name,
            format!("Pid: {:>8}", format!("{}", self.pid)),
            format!("Socket: {:<30} ",
                self.socket_file(config)
                .expect("problem with socket file...")
                .to_str()
                .expect("path has invalid chars")
            ),
        )
    }

    fn socket_file(&self, config: &Config) -> Result<PathBuf, ()> {
        match &self.user_id {
            Some(uid) => {
                let socket_path = PathBuf::from(config.tmp_dir)
                    .join(format!("emacs{}", uid.deref() ))
                    .join(self.socket_name.clone());
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
    }

    fn kill(&self) -> Result<Pid, std::io::Error> {
        let system = System::new_all();
        let pid = self.pid;
        match system.process(pid) {
            // Process should be killed with TERM signal (15),
            // this is consistent with `kill PID` on MacOS and allows
            // the Emacs daemon process to clear up its socket file.
            Some(process) => match process.kill_with(Signal::Term) {
                Some(true) => Ok(pid),
                Some(false) => Err(
                    std::io::Error::new(std::io::ErrorKind::Other,
                    format!("Error trying to send kill signal to Emacs daemon '{}' with Pid {}.", self.socket_name, pid)
                    )
                ),
                None => Err(
                    std::io::Error::new(std::io::ErrorKind::Other, "Signal::Term does not exist on this system.")
                ),
            },
            None => Err(
                std::io::Error::new(std::io::ErrorKind::Other,
                format!("Error trying to send kill signal to Emacs daemon. No process found with with Pid {}.", pid)
                )
            )
        }
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


fn kill_daemon(name: &str) ->  Result<(), std::io::Error> {
    match daemons.iter().find(|&p| p.socket_name == name) {
        Some(daemon) => {
            match daemon.kill() {
                Ok(pid) => {
                    println!("{}", pid);
                    Ok(())
                },
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            }
        },
        None => Err(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("No Emacs daemon found with socket name {}", name)
            )
        ),
    }
}

