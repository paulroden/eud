use std::ops::Deref;
use std::process::{Child, Command, Stdio};
use std::path::{Path, PathBuf};
use std::env;
use sysinfo::{Pid, Process, ProcessExt, Uid, Signal, System, SystemExt};

fn main() {

    let config = Config {
        default_socket: "server",
        tmp_dir: "/tmp/",
    };

    // `list`
    list_daemons(&config).unwrap();

    println!("{:?}", active_daemons_names());

    // `new`
    let new_daemon = launch_emacs_daemon(Some("test-daemon-3"), &config);

    match new_daemon {
        Ok(d) => {
            let output = d.wait_with_output().expect("what? no outouts??");
            println!("stdout:\n{:#?}\n", String::from_utf8_lossy(output.stdout.as_slice()) );
            println!("stderr:\n{:#?}", String::from_utf8_lossy(output.stderr.as_slice()));
        },
        Err(e) => eprintln!("No daemon process started.. wtf?\n{e}"),
    }

    // `connect`
    match launch_client("test-daemon-3", &config) {
        Ok(client) => {
            println!("Launched Emacs client {:?}", client);
            std::thread::sleep(std::time::Duration::from_secs(5));
        },
        Err(e) => eprint!("Error launching client {e}"),
    }
   
    // `kill`
    match kill_daemon("test-daemon-3") {
        Ok(_) => println!("Killed it."),
        Err(e) => {
            eprintln!("{}", e);
            list_daemons(&config).unwrap();
        },
    }

    // ...
    list_daemons(&config).unwrap();
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
    println!("Current Emacs daemon instances:");
    get_daemons().iter().for_each(|daemon| {
        println!("{}", daemon.show(&config));
    });
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
        // The socket name needs to be derived from the command arguments
        // passed to emacs. These will be of the form:
        // --bg-daemon=\xxx,y\012/name//or/socket/path
        // The result of `p.cmd()` is therefore parsed to extract the
        // "/name//or/socket/path" portion into a `Path`, to extract the
        // socket filename 
        let socket_name = Path::new(p.cmd().get(1)?
            .split_once('=')?
            .1
            .split('\n')
            .last()?
        ).file_name()?.to_str();
        
        Some(Self {
            pid: p.pid(),
            user_id: p.user_id().cloned(),
            name: p.name().into(),
            socket_name: socket_name?.to_owned(),
        })
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
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Error trying to send kill signal to Emacs daemon. No process found with with Pid {}.", pid)
                )
            )
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

    fn socket_file(&self, config: &Config) -> Result<PathBuf, std::io::Error> {
        match &self.user_id {
            Some(uid) => {
                let socket_path = PathBuf::from(config.tmp_dir)
                    .join(format!("emacs{}", uid.deref() ))
                    .join(self.socket_name.clone());
                match socket_path.exists() {
                    true => Ok(socket_path),
                    false => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Daemon socket at path {:?} does not exist.", socket_path)
                    )),
                }
            },
            None => Err(std::io::Error::new(std::io::ErrorKind::Other,
                format!("Unexpected! No user ID present for Emacs daemon process:\n{:?}", self)
            )),
        }
    }
}


#[derive(Clone, Debug)]
struct ClientProcess {
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
                match self.create_new_frame {
                    true => format!("--create-frame"),
                    false => format!("--reuse-frame"),
                }
            )
           .spawn()
    }
}



fn launch_client(daemon_name: &str, config: &Config) -> Result<Child, std::io::Error> {
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


/// should return a type which captures either: Child process for a newly-spawned Emacs daemon, or a Process capturing the 
fn launch_emacs_daemon(name: Option<&str>, config: &Config) -> std::io::Result<Child> {
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
// TODO: (above) look into std::process::Commmand::{current_dir, envs}


fn kill_daemon(name: &str) ->  Result<(), std::io::Error> {
    match get_daemons().iter().find(|&p| p.socket_name == name) {
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
