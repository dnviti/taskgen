# Systemd Task Generator (taskgen)

The Systemd Task Generator (`taskgen`) is a versatile shell script designed to simplify the creation and management of systemd timers and services. By abstracting the complexities of systemd's configuration files, `taskgen` provides an easy-to-use interface for scheduling and executing tasks on modern Linux systems.

## Why Systemd over Cron?

Systemd offers several advantages over traditional cron jobs for scheduling tasks:

- **Integrated System Management**: Systemd provides a unified framework for managing system processes, including scheduled tasks, improving consistency and reliability.
- **Flexibility**: Systemd timers support more complex scheduling scenarios than cron, including dependencies on system states or other units.
- **Logging and Monitoring**: Logs for systemd jobs are managed through the journal, offering centralized and comprehensive logging.
- **Resource Control**: Systemd allows for precise control over the resources available to scheduled tasks, ensuring that critical system resources remain unaffected by background tasks.
- **Security**: Enhanced security features, such as sandboxing, are available for tasks managed by systemd.

Below is a sample documentation for the enhanced systemd timer and service creation script, designed to be posted on GitHub. This documentation includes an introduction, why to use systemd instead of cron, installation instructions, usage examples, and explains all possible uses.

---
## Installation

To install the script, use either `wget` or `curl` to download it, and then use `sudo install` to place it in the `/usr/local/bin` directory, making it executable and available system-wide.

```bash
# Using wget
wget https://github.com/dnviti/taskgen/releases/download/latest/taskgen -O taskgen

# Or using curl
curl -O https://github.com/dnviti/taskgen/releases/download/latest/taskgen -o taskgen

# Install the script
sudo install -o root -g root -m 0755 taskgen /usr/local/bin/taskgen
```

## Usage

The script supports creating and deleting systemd timers and services with a variety of options for detailed customization.

### Basic Command Structure

```bash
taskgen --name NAME --command COMMAND [--frequency FREQUENCY] [--operation OPERATION] [--timer-options OPTIONS]
```

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

3. **Specifying Advanced Timer Options**

   Create a timer that starts a task 10 minutes after boot, repeating every 2 hours, with a randomized delay of up to 30 seconds.

   ```bash
   taskgen --name example-task --command "/path/to/script" --timer-options "OnBootSec=10min OnUnitActiveSec=2h RandomizedDelaySec=30s"
   ```

4. **Weekly Email Report**

   Send an email report every Monday at 08:00 AM.

   ```bash
   taskgen --name weekly-email --command "/usr/bin/send-email-report.sh" --frequency weekly
   ```

5. **Reboot System at Specific Time**

   Schedule a system reboot every day at 3:00 AM.

   ```bash
   taskgen --name system-reboot --command "/sbin/reboot" --frequency "*-*-* 03:00:00"
   ```

### Advanced Configuration

For more complex scheduling needs or specific systemd functionality, use the `--timer-options` parameter to directly input systemd timer options. This allows for leveraging the full capability of systemd timers, including dependencies, conditions, and environmental settings.
---

The initial version of this readme and the taskgen script has been generated with GPT-4
