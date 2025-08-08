use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use std::{fs, process::Command};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Task {
    /// Unique identifier for the task, matching the systemd service and timer name in the JSON database.
    name: String,
    /// Command executed by the service; stored as a plain string in the JSON file.
    command: String,
    /// `OnCalendar` expression defining how often the timer runs (e.g., `"daily"`).
    frequency: String,
    /// Comma-separated additional timer directives such as `"Persistent=true"`.
    timer_options: String,
}

fn load_tasks(db_file: &str) -> Vec<Task> {
    match fs::read_to_string(db_file) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_tasks(db_file: &str, tasks: &[Task]) {
    if let Err(e) = fs::write(
        db_file,
        serde_json::to_string_pretty(tasks).unwrap_or_default(),
    ) {
        eprintln!("Failed to write db file: {}", e);
    }
}

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

    let db_file = "/var/lib/taskgen-db.json";

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

    let mut tasks = load_tasks(db_file);
    tasks.push(Task {
        name: name.to_string(),
        command: command.to_string(),
        frequency: frequency.to_string(),
        timer_options: timer_options.to_string(),
    });
    save_tasks(db_file, &tasks);
    println!(
        "Service and timer for {} created and started successfully.",
        name
    );
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

    let tasks = load_tasks(db_file);
    let new_tasks: Vec<Task> = tasks.into_iter().filter(|t| t.name != name).collect();
    save_tasks(db_file, &new_tasks);
    println!("Service and timer for {} deleted successfully.", name);
}

fn list_db(db_file: &str) {
    let tasks = load_tasks(db_file);
    println!("List of systemd timers and services created by taskgen:");
    if tasks.is_empty() {
        println!("No tasks have been created yet.");
    } else {
        for t in tasks {
            println!(
                "{}:{}:{}:{}",
                t.name, t.command, t.frequency, t.timer_options
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn sample_task() -> Task {
        Task {
            name: "sample".to_string(),
            command: "/bin/echo hello".to_string(),
            frequency: "daily".to_string(),
            timer_options: "Persistent=true".to_string(),
        }
    }

    #[test]
    fn task_serialization_round_trip() {
        let task = sample_task();
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task, deserialized);
    }

    #[test]
    fn load_save_round_trip() {
        let task = sample_task();
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();

        save_tasks(&path, &[task.clone()]);
        let loaded = load_tasks(&path);
        assert_eq!(loaded, vec![task]);
    }

    #[test]
    fn load_tasks_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        let tasks = load_tasks(path);
        assert!(tasks.is_empty());
    }

    #[test]
    fn load_tasks_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{ invalid").unwrap();
        let path = file.path().to_str().unwrap();
        let tasks = load_tasks(path);
        assert!(tasks.is_empty());
    }

    #[test]
    fn load_tasks_partial_fields() {
        let mut file = NamedTempFile::new().unwrap();
        file
            .write_all(b"[{\"name\":\"only\"}]")
            .unwrap();
        let path = file.path().to_str().unwrap();
        let tasks = load_tasks(path);
        assert!(tasks.is_empty());
    }
}
