# Systemd Task Generator (taskgen)

The Systemd Task Generator (`taskgen`) is a versatile shell script designed to simplify the creation and management of systemd timers and services. By abstracting the complexities of systemd's configuration files, `taskgen` provides an easy-to-use interface for scheduling and executing tasks on modern Linux systems.

## Why Systemd over Cron?

Systemd offers several advantages over traditional cron jobs for scheduling tasks:

- **Integrated System Management**: Systemd provides a unified framework for managing system processes, including scheduled tasks, improving consistency and reliability.
- **Flexibility**: Systemd timers support more complex scheduling scenarios than cron, including dependencies on system states or other units.
- **Logging and Monitoring**: Logs for systemd jobs are managed through the journal, offering centralized and comprehensive logging.
- **Resource Control**: Systemd allows for precise control over the resources available to scheduled tasks, ensuring that critical system resources remain unaffected by background tasks.
- **Security**: Enhanced security features, such as sandboxing, are available for tasks managed by systemd.

---
## Installation

To install the script, use either `wget` or `curl` to download it, and then use `sudo install` to place it in the `/usr/local/bin` directory, making it executable and available system-wide.

```bash
wget https://github.com/dnviti/taskgen/releases/download/latest/taskgen -O taskgen && \
sudo install -o root -g root -m 0755 taskgen /usr/local/bin/taskgen && \
rm -f taskgen
```

## Usage

The script supports creating and deleting systemd timers and services with a variety of options for detailed customization.
Task metadata is stored in `/var/lib/taskgen-db.json` using a JSON format for improved reliability.

### Task database format

The database file contains a JSON array where each element represents a task object with four string fields:

- `name` – unique identifier matching the generated systemd service and timer.
- `command` – command executed by the service.
- `frequency` – systemd `OnCalendar` expression determining how often the task runs.
- `timer_options` – comma-separated additional timer directives.

Example:

```json
[
  {
    "name": "daily-backup",
    "command": "/usr/bin/backup.sh",
    "frequency": "daily",
    "timer_options": ""
  }
]
```

Unit tests in `src/main.rs` verify serialization, deserialization, and database file handling for this format, ensuring the data model is robust and consistently validated.

### Basic Command Structure

```bash
taskgen --name NAME [--command COMMAND ...] [--frequency FREQUENCY] [--operation OPERATION] [--unit UNIT] [--timer-options OPTIONS] [--create-script SCRIPT]
```

Multiple `--command` options may be supplied to run several commands sequentially. If no `--command` flags are passed, `taskgen` prompts interactively for commands, accepting one per line until a blank line is entered. The `--create-script` option writes the provided commands to a shell script and uses it as the service's `ExecStart`.
The `--operation` flag accepts any `systemctl` operation such as `start`, `stop`, `enable`, `disable`, `restart`, or `status`. Use `--unit` to specify whether the action targets the generated `service` or `timer` (default `timer`).

### Examples

1. **Creating a Daily Backup Task**

   Create a service to perform daily backups at midnight.

   ```bash
   taskgen --name daily-backup --command "/usr/bin/backup.sh" --frequency daily
   ```

2. **Deleting a Task**

   Delete the previously created `daily-backup` task.

   ```bash
   taskgen --name daily-backup --operation delete
   ```

3. **Running Multiple Commands**

   Execute several commands sequentially without writing a script.

   ```bash
   taskgen --name cleanup --command "echo start" --command "rm -rf /tmp/*" --frequency daily
   ```

4. **Creating a Shell Script Automatically**

   Store the provided commands in a script and run that script.

   ```bash
   taskgen --name scripted-task --command "echo one" --command "echo two" --create-script /usr/local/bin/mytask.sh --frequency hourly
   ```

5. **Specifying Advanced Timer Options**

   Create a timer that starts a task 10 minutes after boot, repeating every 2 hours, with a randomized delay of up to 30 seconds.

   ```bash
   taskgen --name example-task --command "/path/to/script" --timer-options "OnBootSec=10min,OnUnitActiveSec=2h,RandomizedDelaySec=30s"
   ```

6. **Weekly Email Report**

   Send an email report every Monday at 08:00 AM.

   ```bash
   taskgen --name weekly-email --command "/usr/bin/send-email-report.sh" --frequency weekly
   ```

7. **Reboot System at Specific Time**

   Schedule a system reboot every day at 3:00 AM.

   ```bash
   taskgen --name system-reboot --command "/sbin/reboot" --frequency "*-*-* 03:00:00"
   ```

8. **Starting a Service Manually**

   Run a systemd operation on the generated service unit instead of the timer.

   ```bash
   taskgen --name daily-backup --operation start --unit service
   ```

### Advanced Configuration

For more complex scheduling needs or specific systemd functionality, use the `--timer-options` parameter to directly input systemd timer options. This allows for leveraging the full capability of systemd timers, including dependencies, conditions, and environmental settings.

---

The initial version of this readme and the taskgen script has been generated with GPT-4 
