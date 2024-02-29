use clap::{App, Arg};
use std::{fs, process::Command, io::Write};

fn main() {
    let matches = App::new("Systemd Timer Manager")
        .version("1.0")
        .author("Your Name <your_email@example.com>")
        .about("Manages systemd timers and services")
        .arg(Arg::with_name("name")
             .short('n')
             .long("name")
             .value_name("NAME")
             .help("Name of the systemd service and timer")
             .takes_value(true))
        .arg(Arg::with_name("command")
             .short('c')
             .long("command")
             .value_name("COMMAND")
             .help("Command that the service will execute")
             .takes_value(true))
        .arg(Arg::with_name("frequency")
             .short('f')
             .long("frequency")
             .value_name("FREQUENCY")
             .help("Frequency of the timer")
             .takes_value(true))
        .arg(Arg::with_name("operation")
             .short('o')
             .long("operation")
             .value_name("OPERATION")
             .help("Operation to perform: create (default) or delete")
             .takes_value(true))
        .arg(Arg::with_name("timer_options")
             .short('t')
             .long("timer-options")
             .value_name("TIMER_OPTIONS")
             .help("Additional systemd timer options")
             .takes_value(true))
        .arg(Arg::with_name("list")
             .short('l')
             .long("list")
             .help("List all created timers and services"))
        .get_matches();

    let db_file = "/var/lib/systemd-timer-db.txt";

    if matches.is_present("list") {
        list_db(db_file);
        return;
    }

    let name = matches.value_of("name").expect("Name is required");
    let operation = matches.value_of("operation").unwrap_or("create");

    match operation {
        "create" => {
            let command = matches.value_of("command").expect("Command is required for create operation");
            let frequency = matches.value_of("frequency").unwrap_or_default();
            let timer_options = matches.value_of("timer_options").unwrap_or_default();
            create_task(name, command, frequency, timer_options, db_file);
        },
        "delete" => delete_task(name, db_file),
        _ => println!("Invalid operation: {}", operation),
    }
}

fn create_task(name: &str, command: &str, frequency: &str, timer_options: &str, db_file: &str) {
    let service_content = format!(
        "[Unit]\nDescription=Service for {}\n\n[Service]\nType=oneshot\nExecStart={}\n",
        name, command
    );
    fs::write(format!("/etc/systemd/system/{}.service", name), service_content).expect("Failed to write service file");

    let mut timer_content = format!(
        "[Unit]\nDescription=Timer for {}\n\n[Timer]\n",
        name
    );
    if !frequency.is_empty() {
        timer_content += &format!("OnCalendar={}\n", frequency);
    }
    if !timer_options.is_empty() {
        let options: Vec<&str> = timer_options.split(',').collect();
        for option in options {
            timer_content += &format!("{}\n", option);
        }
    }
    timer_content += "Persistent=true\n\n[Install]\nWantedBy=timers.target\n";
    fs::write(format!("/etc/systemd/system/{}.timer", name), timer_content).expect("Failed to write timer file");

    Command::new("systemctl").arg("daemon-reload").status().expect("Failed to reload systemd daemon");
    Command::new("systemctl").arg("enable").arg(format!("{}.timer", name)).status().expect("Failed to enable timer");
    Command::new("systemctl").arg("start").arg(format!("{}.timer", name)).status().expect("Failed to start timer");

    let mut db = fs::OpenOptions::new().append(true).create(true).open(db_file).expect("Failed to open db file");
    writeln!(db, "{}:{}:{}:{}", name, command, frequency, timer_options).expect("Failed to write to db file");

    println!("Service and timer for {} created and started successfully.", name);
}

fn delete_task(name: &str, db_file: &str) {
    Command::new("systemctl").arg("stop").arg(format!("{}.timer", name)).status().expect("Failed to stop timer");
    Command::new("systemctl").arg("disable").arg(format!("{}.timer", name)).status().expect("Failed to disable timer");
    fs::remove_file(format!("/etc/systemd/system/{}.service", name)).expect("Failed to remove service file");
    fs::remove_file(format!("/etc/systemd/system/{}.timer", name)).expect("Failed to remove timer file");
    Command::new("systemctl").arg("daemon-reload").status().expect("Failed to reload systemd daemon");

    let contents = fs::read_to_string(db_file).expect("Failed to read db file");
    let new_contents: String = contents.lines().filter(|line| !line.starts_with(name)).collect::<Vec<&str>>().join("\n");
    fs::write(db_file, new_contents).expect("Failed to write updated db file");

    println!("Service and timer for {} deleted successfully.", name);
}

fn list_db(db_file: &str) {
    match fs::read_to_string(db_file) {
        Ok(contents) => {
            println!("List of systemd timers and services created by the script:");
            if contents.is_empty() {
                println!("No tasks have been created yet.");
            } else {
                println!("{}", contents);
            }
        },
        Err(_) => println!("No tasks have been created yet."),
    }
}
