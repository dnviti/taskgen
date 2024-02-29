#!/bin/bash

# Initialize variables
NAME=""
COMMAND=""
FREQUENCY=""
OPERATION="create"  # Default operation
TIMER_OPTIONS=""  # For additional timer options
LIST=false
DB_FILE="/var/lib/systemd-timer-db.txt"  # Path to the database file

# Function to display usage
usage() {
    echo "Usage: $0 --name <name> --command <command> [--frequency <frequency>] [--operation <operation>] [--timer-options <options>] [--list]"
    echo "   or: $0 -n <name> -c <command> [-f <frequency>] [-o <operation>] [-t <options>] [-l]"
    echo
    echo "Options:"
    echo "  -n, --name          Name of the systemd service and timer."
    echo "  -c, --command       Command that the service will execute."
    echo "  -f, --frequency     Frequency of the timer (optional if --timer-options is used)."
    echo "  -o, --operation     Operation to perform: create (default) or delete."
    echo "  -t, --timer-options Additional systemd timer options separated by commas (e.g., 'OnBootSec=10min,OnUnitActiveSec=2h,RandomizedDelaySec=30s')."
    echo "  -l, --list          List all created timers and services."
    exit 1
}

# Function to update the database
update_db() {
    if [ "$OPERATION" == "create" ]; then
        echo "$NAME:$COMMAND:$FREQUENCY:$TIMER_OPTIONS" | sudo tee -a $DB_FILE > /dev/null
    elif [ "$OPERATION" == "delete" ]; then
        sudo sed -i "/^$NAME:/d" $DB_FILE
    fi
}

# Function to list entries from the database
list_db() {
    if [ -f "$DB_FILE" ]; then
        echo "List of systemd timers and services created by the script:"
        cat "$DB_FILE"
    else
        echo "No tasks have been created yet."
    fi
    exit 0
}

# Parse command-line options
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -n|--name) NAME="$2"; shift ;;
        -c|--command) COMMAND="$2"; shift ;;
        -f|--frequency) FREQUENCY="$2"; shift ;;
        -o|--operation) OPERATION="$2"; shift ;;
        -t|--timer-options) TIMER_OPTIONS="$2"; shift ;;
        -l|--list) LIST=true ;;
        *) usage ;;
    esac
    shift
done

# List tasks if requested
if $LIST; then
    list_db
fi

# Check if name is provided for create or delete operations
if [ "$OPERATION" != "list" ] && [ -z "$NAME" ]; then
    usage
fi

# Define file paths
SERVICE_FILE="/etc/systemd/system/${NAME}.service"
TIMER_FILE="/etc/systemd/system/${NAME}.timer"

# Perform the requested operation
case $OPERATION in
    create)
        if [ -z "$COMMAND" ]; then
            usage
        fi

        # Create the service file
        echo "[Unit]
Description=Service for $NAME

[Service]
Type=oneshot
ExecStart=$COMMAND
" | sudo tee $SERVICE_FILE

        # Prepare and write the timer file with proper new lines
        TIMER_CONTENT="[Unit]
Description=Timer for $NAME

[Timer]
"

        if [ -n "$FREQUENCY" ]; then
            TIMER_CONTENT+="OnCalendar=$FREQUENCY\n"
        fi

        IFS=',' read -ra OPTIONS <<< "$TIMER_OPTIONS"
        for OPTION in "${OPTIONS[@]}"; do
            TIMER_CONTENT+="$OPTION\n"
        done

        TIMER_CONTENT+="Persistent=true

[Install]
WantedBy=timers.target
"
        echo -e "$TIMER_CONTENT" | sudo tee $TIMER_FILE > /dev/null

        # Enable and start the timer
        sudo systemctl daemon-reload
        sudo systemctl enable "${NAME}.timer"
        sudo systemctl start "${NAME}.timer"

        # Update the database
        update_db

        echo "Service and timer for $NAME created and started successfully."
        ;;
    
    delete)
        # Stop and disable the timer
        sudo systemctl stop "${NAME}.timer"
        sudo systemctl disable "${NAME}.timer"

        # Remove the service and timer files
        sudo rm -f $SERVICE_FILE $TIMER_FILE

        # Reload systemd to apply changes
        sudo systemctl daemon-reload

        # Update the database
        update_db

        echo "Service and timer for $NAME deleted successfully."
        ;;
    
    *)
        echo "Invalid operation: $OPERATION"
        usage
        ;;
esac
