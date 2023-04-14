use clap::{Args, Parser, Subcommand, ValueEnum};
use super::config::{Config};
use super::daemons::{list_daemons, launch_daemon, kill_daemon};
use super::clients::launch_client;


#[derive(Debug, Parser)]
#[command(name = "eud")]
#[command(about = "manage Emacs clients and daemons", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Debug, Subcommand)]
enum Commands {
    /// List active daemons
    #[command()]
    List,

    #[command(arg_required_else_help = true)]
    New {
        name: String,
    },

    #[command(arg_required_else_help = true)]
    Kill {
        daemon: String,
    },

    #[command(arg_required_else_help = true)]
    Connect {
        daemon: String,
    },
}


pub fn cli(config: &Config) -> Result<(), std::io::Error> {
    let cl = Cli::parse();

    match &cl.command {
        Commands::List => {
            list_daemons(&config)?;
        },
        Commands::New{ name } => {
            let new_daemon = launch_daemon(Some(name), &config);

            match new_daemon {
                Ok(d) => {
                    let output = d.wait_with_output().expect("what? no outouts??");
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
        Commands::Kill{ daemon } => {
            match kill_daemon(daemon) {
                Ok(_) => println!("Killed it."),
                Err(e) => {
                    eprintln!("{}", e);
                    list_daemons(&config).unwrap();
                },
            }
        },
        Commands::Connect{ daemon } => {
            match launch_client(daemon, &config) {
                Ok(client) => {
                    println!("Launched Emacs client {:?}", client);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                },
                Err(e) => eprint!("Error launching client {e}"),
            }
        }
        
    }

    Ok(())
}
