use eud::config::Config;
use eud::clients::launch_client;
use eud::daemons::{list_daemons, active_daemons_names, launch_daemon, kill_daemon};


fn main() {

    let config = Config::new("server", "/tmp/");

    // `list`
    list_daemons(&config).unwrap();

    println!("{:?}", active_daemons_names());

    // `new`
    let new_daemon = launch_daemon(Some("test-daemon-3"), &config);

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

