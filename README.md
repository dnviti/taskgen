# Systemd Task Generator (taskgen)

The Systemd Task Generator (`taskgen`) is a versatile shell script designed to simplify the creation and management of systemd timers and services. By abstracting the complexities of systemd's configuration files, `taskgen` provides an easy-to-use interface for scheduling and executing tasks on modern Linux systems.

## Why Systemd over Cron?

Systemd offers several advantages over traditional cron jobs for scheduling tasks:

- **Integrated System Management**: Systemd provides a unified framework for managing system processes, including scheduled tasks, improving consistency and reliability.
- **Flexibility**: Systemd timers support more complex scheduling scenarios than cron, including dependencies on system states or other units.
- **Logging and Monitoring**: Logs for systemd jobs are managed through the journal, offering centralized and comprehensive logging.
- **Resource Control**: Systemd allows for precise control over the resources available to scheduled tasks, ensuring that critical system resources remain unaffected by background tasks.
- **Security**: Enhanced security features, such as sandboxing, are available for tasks managed by systemd.

## Installation

To install `taskgen`, you can use `wget` or `curl` to download the script from this repository, then use the `install` command to place it in your system's binary directory, making it executable by root. This ensures that the script can be easily executed from anywhere on your system.

### Using wget

```bash
wget [URL_TO_TASKGEN_SCRIPT] -O taskgen
sudo install -o root -g root -m 0755 taskgen /usr/local/bin/taskgen
```

### Using curl

```bash
curl -o taskgen [URL_TO_TASKGEN_SCRIPT]
sudo install -o root -g root -m 0755 taskgen /usr/local/bin/taskgen
```

Replace `[URL_TO_TASKGEN_SCRIPT]` with the actual URL where the `taskgen` script is hosted.

## Usage

### Creating a New Timer

```bash
taskgen --name <name> --command <command> [--frequency <frequency>] [--timer-options <options>] [--operation create]
```

### Deleting an Existing Timer

```bash
taskgen --name <name> --operation delete
```

### Parameters

- `-n, --name`: Name of the systemd service and timer.
- `-c, --command`: Command that the service will execute.
- `-f, --frequency`: (Optional) Frequency of the timer (e.g., `daily`, `weekly`, `@reboot`). This is optional if `--timer-options` is provided.
- `-o, --operation`: (Optional) Operation to perform: `create` (default) or `delete`.
- `-t, --timer-options`: (Optional) Additional systemd timer options (e.g., `OnBootSec=10min`). This provides advanced customization for the timer.

## Examples

### Example 1: Create a Daily Timer

To create a timer that runs a backup script daily at midnight:

```bash
taskgen --name daily-backup --command "/usr/local/bin/backup.sh" --frequency daily
```

### Example 2: Delete a Timer

To delete the previously created `daily-backup` timer:

```bash
taskgen --name daily-backup --operation delete
```

## Advanced Configuration

For complex scheduling needs, use the `--timer-options` parameter to specify custom systemd timer directives. Consult the systemd.timer man page (`man systemd.timer`) for a comprehensive list of available options.

---

The initial version of this readme and the taskgen script has been generated with GPT-4
