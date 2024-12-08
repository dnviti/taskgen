use clap::{Arg, ArgAction, Command};
use configparser::ini::Ini;
use std::{fs, io::Write, process::Command as ProcCommand};

fn main() {
    // Load configuration
    let config = match load_config("/etc/taskgen/taskgen.conf") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to read configuration file: {}", e);
            return;
        }
    };

    // Fetch configuration values with defaults
    let db_file = config.get("DEFAULT", "db_file").unwrap_or("/var/lib/taskgen-db".to_string());
    let systemd_unit_dir = config
        .get("DEFAULT", "systemd_unit_dir")
        .unwrap_or("/etc/systemd/system".to_string());

    // Command-line argument parsing
    let matches = Command::new("Systemd Timer Manager")
        .version("0.1")
        .author("Daniele Viti <dnviti@gmail.com>")
        .about("Manages systemd timers and services")
        .arg_required_else_help(true)
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Name of the systemd service and timer")
                .num_args(1)
                .required_unless_present("list"),
        )
        .arg(
            Arg::new("command")
                .short('c')
                .long("command")
                .value_name("COMMAND")
                .help("Command that the service will execute")
                .num_args(1),
        )
        .arg(
            Arg::new("frequency")
                .short('f')
                .long("frequency")
                .value_name("FREQUENCY")
                .help("Frequency of the timer")
                .num_args(1)
                .default_value("daily"),
        )
        .arg(
            Arg::new("operation")
                .short('o')
                .long("operation")
                .value_name("OPERATION")
                .help("Operation to perform: create (default) or delete")
                .num_args(1)
                .default_value("create"),
        )
        .arg(
            Arg::new("timer_options")
                .short('t')
                .long("timer-options")
                .value_name("TIMER_OPTIONS")
                .help("Additional systemd timer options, comma-separated")
                .num_args(1),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List all created timers and services")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("list") {
        list_db(&db_file);
        return;
    }

    let name = matches.get_one::<String>("name").expect("Name is required");
    let operation = matches.get_one::<String>("operation").unwrap().as_str();

    match operation {
        "create" => {
            let command = matches
                .get_one::<String>("command")
                .expect("Command is required for create operation");
            let frequency = matches.get_one::<String>("frequency").unwrap().as_str();
            let timer_options = matches
                .get_one::<String>("timer_options")
                .map(String::as_str)
                .unwrap_or("");
            create_task(
                name,
                command,
                frequency,
                timer_options,
                &db_file,
                &systemd_unit_dir,
            );
        }
        "delete" => delete_task(name, &db_file, &systemd_unit_dir),
        _ => eprintln!("Invalid operation: {}", operation),
    }
}

fn load_config(path: &str) -> Result<Ini, Box<dyn std::error::Error>> {
    let mut config = Ini::new();
    config.load(path)?;
    Ok(config)
}

fn create_task(
    name: &str,
    command: &str,
    frequency: &str,
    timer_options: &str,
    db_file: &str,
    systemd_unit_dir: &str,
) {
    let service_file = format!("{}/{}.service", systemd_unit_dir, name);
    let timer_file = format!("{}/{}.timer", systemd_unit_dir, name);

    // Write service file
    if let Err(e) = fs::write(
        &service_file,
        format!(
            "[Unit]
Description=Service for {}

[Service]
Type=oneshot
ExecStart={}
",
            name, command
        ),
    ) {
        eprintln!("Failed to write service file: {}", e);
        return;
    }

    // Build timer content
    let mut timer_content = format!(
        "[Unit]
Description=Timer for {}

[Timer]
OnCalendar={}
",
        name, frequency
    );

    if !timer_options.is_empty() {
        for option in timer_options.split(',') {
            timer_content.push_str(option);
            timer_content.push('\n');
        }
    }

    timer_content.push_str(
        "Persistent=true

[Install]
WantedBy=timers.target
",
    );

    // Write timer file
    if let Err(e) = fs::write(&timer_file, timer_content) {
        eprintln!("Failed to write timer file: {}", e);
        return;
    }

    // Reload systemd daemon and enable/start timer
    if !run_systemctl(&["daemon-reload"])
        || !run_systemctl(&["enable", &format!("{}.timer", name)])
        || !run_systemctl(&["start", &format!("{}.timer", name)])
    {
        return;
    }

    // Update database
    if let Err(e) = append_to_db(db_file, name, command, frequency, timer_options) {
        eprintln!("Failed to update db file: {}", e);
        return;
    }

    println!("Service and timer for '{}' created and started successfully.", name);
}

fn delete_task(name: &str, db_file: &str, systemd_unit_dir: &str) {
    // Stop and disable timer
    if !run_systemctl(&["stop", &format!("{}.timer", name)])
        || !run_systemctl(&["disable", &format!("{}.timer", name)])
    {
        return;
    }

    // Remove service and timer files
    let service_file = format!("{}/{}.service", systemd_unit_dir, name);
    let timer_file = format!("{}/{}.timer", systemd_unit_dir, name);
    for file in &[&service_file, &timer_file] {
        if let Err(e) = fs::remove_file(file) {
            eprintln!("Failed to remove {}: {}", file, e);
        }
    }

    // Reload systemd daemon
    if !run_systemctl(&["daemon-reload"]) {
        return;
    }

    // Update database
    if let Err(e) = remove_from_db(db_file, name) {
        eprintln!("Failed to update db file: {}", e);
        return;
    }

    println!("Service and timer for '{}' deleted successfully.", name);
}

fn run_systemctl(args: &[&str]) -> bool {
    match ProcCommand::new("systemctl").args(args).status() {
        Ok(status) if status.success() => true,
        Ok(status) => {
            eprintln!("systemctl {:?} failed with exit code: {}", args, status);
            false
        }
        Err(e) => {
            eprintln!("Failed to execute systemctl {:?}: {}", args, e);
            false
        }
    }
}

fn append_to_db(
    db_file: &str,
    name: &str,
    command: &str,
    frequency: &str,
    timer_options: &str,
) -> std::io::Result<()> {
    let mut db = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(db_file)?;
    writeln!(db, "{}:{}:{}:{}", name, command, frequency, timer_options)?;
    Ok(())
}

fn remove_from_db(db_file: &str, name: &str) -> std::io::Result<()> {
    let contents = fs::read_to_string(db_file)?;
    let new_contents = contents
        .lines()
        .filter(|line| !line.starts_with(&format!("{}:", name)))
        .collect::<Vec<&str>>()
        .join("\n");
    fs::write(db_file, new_contents)?;
    Ok(())
}

fn list_db(db_file: &str) {
    match fs::read_to_string(db_file) {
        Ok(contents) => {
            println!("List of systemd timers and services created by taskgen:");
            if contents.trim().is_empty() {
                println!("No tasks have been created yet.");
            } else {
                for line in contents.lines() {
                    let parts: Vec<&str> = line.splitn(4, ':').collect();
                    if parts.len() >= 3 {
                        println!(
                            "Name: {}, Command: {}, Frequency: {}, Options: {}",
                            parts[0],
                            parts[1],
                            parts[2],
                            parts.get(3).unwrap_or(&"")
                        );
                    }
                }
            }
        }
        Err(_) => println!("No tasks have been created yet."),
    }
}