use std::path::PathBuf;
use clap::{Parser, Subcommand};
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
    List,

    /// launch new daemon
    #[command(arg_required_else_help = true)]
    New {
        name: String,
    },

    /// kill daemon with socket NAME, or kill all active daemons with --all
    #[command(arg_required_else_help = true)]
    Kill {
        daemon_name: String,
    },
    
    /// connect Emacs client to daemon; visits path at FILE
    #[command(arg_required_else_help = true)]
    Connect {
        #[arg(required = true)]
        daemon: String,
        #[arg(required = false)]
        file: Option<PathBuf>,
    },
}


pub fn cli(config: &Config) -> Result<(), std::io::Error> {

    match &Cli::parse().command {
        Commands::List => {
            list_daemons(&config)?;
        },
        Commands::New{ name } => {
            let new_daemon = daemons::launch_new(Some(name), &config);

            match new_daemon {
                Ok(daemon) => {
                    let output = daemon.wait_with_output().expect("what? no outouts??");
                    println!(
                        "stdout:\n{:#?}\n",
                        String::from_utf8_lossy(output.stdout.as_slice())
                    );
                    println!(
                        "stderr:\n{:#?}",
                        String::from_utf8_lossy(output.stderr.as_slice())
                    );
                },
                Err(e) => eprintln!("No daemon process started.. wtf?\n{e}"),
            }
        },
        Commands::Kill{ daemon_name } => {
            match daemons::kill_by_name(daemon_name) {
                Ok(pid) => println!(
                    "Killed Emacs daemon '{}' [Pid: {} ]",
                    daemon_name,
                    pid
                ),
                Err(e) => {
                    eprintln!("{}", e);
                    list_daemons(&config)?;
                },
            }                
        }
        Commands::Connect{ daemon, file } => {
            let visit_file = file.clone().unwrap_or(std::env::current_dir()?);
            match client::connect(daemon, visit_file, &config) {
                Ok(client) => {
                    println!("Launched Emacs client {:?}", client);
                    let output = client.wait_with_output().expect("what? no outputs??");
                    println!(
                        "stdout:\n{:#?}\n",
                        String::from_utf8_lossy(output.stdout.as_slice())
                    );
                    println!(
                        "stderr:\n{:#?}",
                        String::from_utf8_lossy(output.stderr.as_slice())
                    );
                },
                Err(e) => eprint!("Error launching client:\n{e}"),
            }
        }
    }
    Ok(())
}


pub fn list_daemons(config: &Config) -> Result<(), std::io::Error> {
    println!("Current Emacs daemon instances:");
    daemons::get_all().iter().for_each(|daemon| {
        println!("{}", daemon.show(&config));
    });
    Ok(())
}
