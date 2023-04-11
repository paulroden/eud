use std::fs;
use std::process::{Child, Command};
use std::path::{Path, PathBuf};
use std::env;
use sysinfo::{Pid, Process, ProcessExt, Uid, System, SystemExt};
use users::get_current_uid;

fn main() {

    let config = Config {
        default_socket_name: "server-3",
        default_tmp_dir: "/tmp/",
    };


    println!("All the sockets are in: {:?}", sockets_dir(&config));
    // println!("Temp Dir is: {:?}", temp_dir(&config));
    // println!("You are: {:?}", get_current_uid());
    //let daemeon = launch_emacs_daemon(None, &config);
    //println!("{:?}", daemeon);

    println!("Daemon sockets are");

    for item in &daemon_sockets(&config) {
        println!("{:#?}", item);
    }

    
    // let s = System::new_all();
    // println!("emacs|Emacs processes");
    // let running_daemons =
    //     s.processes_by_name("emacs")
    //         .chain(s.processes_by_name("Emacs"))
    //         .map(|p| DaemonProcess::from_sys_process(p))
    //         .collect::<Vec<_>>();

    // println!("{:#?}", running_daemons);

    let state = State::get();
    println!("All clients: {:#?}", state.all_clients() );
    println!("All daemons: {:#?}", state.all_daemons() );
    

}


#[derive(Debug)]
struct State {
    emacs_processes: Vec<EmacsProcess>,
}

impl State {
    fn get() -> Self {
        let system = System::new_all();
        Self {
            emacs_processes: system.processes()
                .iter().filter(|(_, p)| p.name()
                    .to_lowercase()
                    .starts_with("emacs")
                )
                .map(|(_, p)| EmacsProcess::from_sys_process(p))
                .collect::<Vec<_>>()
        }
    }

    fn all_clients(&self) -> Vec<EmacsProcess> {
        self.emacs_processes.iter().filter(|p| p.is_client() ).cloned().collect()
    }

    fn all_daemons(&self) -> Vec<EmacsProcess> {
        self.emacs_processes.iter().filter(|p| p.is_daemon()).cloned().collect()
    }
    
}



#[derive(Debug)]
struct Config {
    default_socket_name: &'static str,
    default_tmp_dir: &'static str,
}


#[derive(Debug)]
struct Socket {
    file: PathBuf,
}



#[derive(Clone, Debug)]
struct EmacsProcess {
    pid: Pid,
    user_id: Option<Uid>,
    name: String,
    executable: PathBuf,
    command: Vec<String>,
    cwd: PathBuf,
}

impl EmacsProcess {
    fn is_client(&self) -> bool {
        self.name == "emacsclient"
    }

    fn is_daemon(&self) -> bool {
        match self.command.get(1) {
            None => false,
            Some(args) => args.contains("daemon"),
        }
    }
    
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
}



fn sockets_dir(config: &Config) -> PathBuf {
    PathBuf::from(
        env::var("TMPDIR")
            .unwrap_or(config.default_tmp_dir.to_string())
    ).join(
        format!("emacs{}", get_current_uid())
    )
}


fn daemon_sockets(config: &Config) -> Result<Vec<PathBuf>, std::io::Error> {
    fs::read_dir(&sockets_dir(config))?
        .map(|item| item.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
}


/// should return a type which captures either: Child process for a newly-spawned Emacs daemon, or a Process capturing the 
fn launch_emacs_daemon(name: Option<&str>, config: &Config) -> std::io::Result<Child> {
    let daemon_name = match name {
        Some(name) => name,
        None => &config.default_socket_name,
    };
    Command::new("emacs")
        .arg(format!("--daemon={}", daemon_name))
        .spawn()
}

fn emacs_daemon_processes() {
    unimplemented!()
}

/*
 clienten
   daemon | -d
     new [NAME]
     Create a new daemon process. If nothing passed, use default name ("server"); if NAME passed, use this as the socket name.
     kill NAME | --all
       kill emacs server NAME; or kill all emacs processes with --all
     show
       list active Emacs daemons


*/
