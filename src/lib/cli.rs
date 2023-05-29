use std::path::PathBuf;
use clap::{Parser, Subcommand};
use standard_styled::standard_styled;
use super::config::Config;
use super::daemons;
use super::client;



#[derive(Debug, Parser)]
#[command(name = "eud")]
#[command(about = "manage Emacs clients and daemons", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Debug, Subcommand)]
enum Commands {
    
    /// list active daemons
    #[command()]
    List {
        /// list active daemons's names only, one per line
        #[arg(short = '1', default_value_t = false)]
        short: bool,
    },

    /// launch new daemon
    #[command()]
    New {
        name: Option<String>,
    },

    /// kill daemon with socket NAME, or kill all active daemons with --all
    #[command(arg_required_else_help = true)]
    Kill {
        #[arg(long = "all", default_value_t = false)]
        all: bool,
        daemon_name: Option<String>,
    },
    
    /// connect Emacs client to daemon; visits path at FILE
    #[command(arg_required_else_help = true)]
    Connect {
        #[arg(required = true)]
        daemon: String,
        #[arg(required = false)]
        file: Option<PathBuf>,
    },

    /// print directory location for daemon socket files (can be passed to Emacs' `server-socket-dir' variable)
    #[command()]
    ServerSocketDirPath
}


pub fn cli(config: &Config) -> Result<(), std::io::Error> {

    match &Cli::parse().command {
        Commands::List { short } => {
            match short {
                true  => list_daemons_short(),
                false => list_daemons(&config)?,
            }
        },
        Commands::New{ name } => {
            let name_or_default = name.clone()
                .unwrap_or(config.default_socket_name().clone());
            // first check if a daemon with the same socket name (or the
            // default name) already exists (whether in `eud's
            // `server_socket_dir` location or otherwise)
            match daemons::active_daemons_names().contains(&name_or_default) {
                true => println!("A daemon with name '{name_or_default}' is already running. If you wish to connect to it, try:\n    `eud connect {name_or_default} [FILE]`"),
                false => {
                    tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()?
                        .block_on(async {
                            let name = name.clone();
                            let cmd = daemons::build_new(name, config);
                            match standard_styled(cmd, config.style()).await {
                                Ok(_) => (),
                                Err(e) => eprintln!("Tokio error from `standard_styled: {e}"),
                            }
                        })
                }
            }
        },
        Commands::Kill{ all, daemon_name } => {
            if *all {
                for result in daemons::kill_all() {
                    match result {
                        Ok(pid) => println!("Killed Emacs daemon with Pid {}", pid),
                        Err(e)  => eprintln!(
                            "Error trying to kill Emacs daemon process:\n{e}"
                        ),
                    }
                }
            } else {
                if let Some(name) = daemon_name {
                    match daemons::kill_by_name(name) {
                        Ok(pid) => println!("Killed Emacs daemon '{name}' [Pid: {pid} ]"),
                        Err(e) => {
                            eprintln!("{}", e);
                            list_daemons(&config)?;
                        },
                    }
                }
            }
        },
        Commands::Connect{ daemon, file } => {
            let visit_file = file.clone().unwrap_or(std::env::current_dir()?);
            match client::connect(daemon, visit_file, &config) {
                Ok(client) => {
                    println!("Launching Emacs client connected to '{}' ...", daemon);
                    let output = client
                        .wait_with_output().expect("what? no outputs??");
                    println!(
                        "stdout:\n{}\n",
                        String::from_utf8_lossy(output.stdout.as_slice())
                    );
                    println!(
                        "stderr:\n{}",
                        String::from_utf8_lossy(output.stderr.as_slice())
                    );
                },
                Err(e) => eprint!("Error launching client:\n{e}"),
            }
        },
        Commands::ServerSocketDirPath => {
            print!("{}", config.server_socket_dir().display());
        }
    }
    Ok(())
}


pub fn list_daemons(config: &Config) -> Result<(), std::io::Error> {
    let extant_daemons = daemons::get_all();
    match extant_daemons.len() {
        0 => println!("No Emacs daemon processes are running."),
        _ => {
            println!("Current Emacs daemon instances:");
            extant_daemons.iter().for_each(|daemon| {
                println!("{}", daemon.show(&config));
            });
        }
    }
    Ok(())
}

pub fn list_daemons_short() -> () {
    daemons::active_daemons_names().iter()
        .for_each(|name| println!("{name}"))
}


