use clap::{App, Arg};
use std::{fs, io::Write, process::Command};

fn main() {
    let matches = App::new("Systemd Timer Manager")
        .version("0.1")
        .author("Daniele Viti <dnviti@gmail.com>")
        .about("Manages systemd timers and services")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Name of the systemd service and timer")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("command")
                .short('c')
                .long("command")
                .value_name("COMMAND")
                .help("Command that the service will execute")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("frequency")
                .short('f')
                .long("frequency")
                .value_name("FREQUENCY")
                .help("Frequency of the timer")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("operation")
                .short('o')
                .long("operation")
                .value_name("OPERATION")
                .help("Operation to perform: create (default) or delete")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timer_options")
                .short('t')
                .long("timer-options")
                .value_name("TIMER_OPTIONS")
                .help("Additional systemd timer options")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("list")
                .short('l')
                .long("list")
                .help("List all created timers and services"),
        )
        .get_matches();

    let db_file = "/var/lib/taskgen-db";

    if matches.is_present("list") {
        list_db(db_file);
        return;
    }

    let name = matches.value_of("name").expect("Name is required");
    let operation = matches.value_of("operation").unwrap_or("create");

    match operation {
        "create" => {
            let command = matches
                .value_of("command")
                .expect("Command is required for create operation");
            let frequency = matches.value_of("frequency").unwrap_or_default();
            let timer_options = matches.value_of("timer_options").unwrap_or_default();
            create_task(name, command, frequency, timer_options, db_file);
        }
        "delete" => delete_task(name, db_file),
        _ => println!("Invalid operation: {}", operation),
    }
}

fn create_task(name: &str, command: &str, frequency: &str, timer_options: &str, db_file: &str) {
    let service_content = format!(
        "[Unit]\nDescription=Service for {}\n\n[Service]\nType=oneshot\nExecStart={}\n",
        name, command
    );

    if let Err(e) = fs::write(
        format!("/etc/systemd/system/{}.service", name),
        service_content,
    ) {
        eprintln!("Failed to write service file: {}", e);
        return;
    }

    let mut timer_content = format!("[Unit]\nDescription=Timer for {}\n\n[Timer]\n", name);
    if !frequency.is_empty() {
        timer_content += &format!("OnCalendar={}\n", frequency);
    }
    if !timer_options.is_empty() {
        timer_options.split(',').for_each(|option| {
            timer_content += &format!("{}\n", option);
        });
    }
    timer_content += "Persistent=true\n\n[Install]\nWantedBy=timers.target\n";

    if let Err(e) = fs::write(format!("/etc/systemd/system/{}.timer", name), timer_content) {
        eprintln!("Failed to write timer file: {}", e);
        return;
    }

    let reload_status = Command::new("systemctl").arg("daemon-reload").status();
    if let Err(e) = reload_status {
        eprintln!("Failed to reload systemd daemon: {}", e);
        return;
    }

    let enable_status = Command::new("systemctl")
        .arg("enable")
        .arg(format!("{}.timer", name))
        .status();
    if let Err(e) = enable_status {
        eprintln!("Failed to enable timer: {}", e);
        return;
    }

    let start_status = Command::new("systemctl")
        .arg("start")
        .arg(format!("{}.timer", name))
        .status();
    if let Err(e) = start_status {
        eprintln!("Failed to start timer: {}", e);
        return;
    }

    match fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(db_file)
    {
        Ok(mut db) => {
            if let Err(e) = writeln!(db, "{}:{}:{}:{}", name, command, frequency, timer_options) {
                eprintln!("Failed to write to db file: {}", e);
            } else {
                println!(
                    "Service and timer for {} created and started successfully.",
                    name
                );
            }
        }
        Err(e) => eprintln!("Failed to open db file: {}", e),
    }
}

fn delete_task(name: &str, db_file: &str) {
    let stop_status = Command::new("systemctl")
        .arg("stop")
        .arg(format!("{}.timer", name))
        .status();
    if let Err(e) = stop_status {
        eprintln!("Failed to stop timer: {}", e);
        return;
    }

    let disable_status = Command::new("systemctl")
        .arg("disable")
        .arg(format!("{}.timer", name))
        .status();
    if let Err(e) = disable_status {
        eprintln!("Failed to disable timer: {}", e);
        return;
    }

    if let Err(e) = fs::remove_file(format!("/etc/systemd/system/{}.service", name)) {
        eprintln!("Failed to remove service file: {}", e);
    }

    if let Err(e) = fs::remove_file(format!("/etc/systemd/system/{}.timer", name)) {
        eprintln!("Failed to remove timer file: {}", e);
    }

    let reload_status = Command::new("systemctl").arg("daemon-reload").status();
    if let Err(e) = reload_status {
        eprintln!("Failed to reload systemd daemon: {}", e);
        return;
    }

    match fs::read_to_string(db_file) {
        Ok(contents) => {
            let new_contents: String = contents
                .lines()
                .filter(|line| !line.starts_with(name))
                .collect::<Vec<&str>>()
                .join("\n");
            if let Err(e) = fs::write(db_file, new_contents) {
                eprintln!("Failed to write updated db file: {}", e);
            } else {
                println!("Service and timer for {} deleted successfully.", name);
            }
        }
        Err(e) => eprintln!("Failed to read db file: {}", e),
    }
}

fn list_db(db_file: &str) {
    match fs::read_to_string(db_file) {
        Ok(contents) => {
            println!("List of systemd timers and services created by taskgen:");
            if contents.is_empty() {
                println!("No tasks have been created yet.");
            } else {
                println!("{}", contents);
            }
        }
        Err(_) => println!("Failed to read db file or no tasks have been created yet."),
    }
}
