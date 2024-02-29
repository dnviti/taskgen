#!/bin/bash

# Initialize variables
NAME=""
COMMAND=""
FREQUENCY=""
OPERATION="create"  # Default operation
TIMER_OPTIONS=""  # For additional timer options

# Function to display usage
usage() {
    echo "Usage: $0 --name <name> --command <command> [--frequency <frequency>] [--operation <operation>] [--timer-options <options>]"
    echo "   or: $0 -n <name> -c <command> [-f <frequency>] [-o <operation>] [-t <options>]"
    echo
    echo "Options:"
    echo "  -n, --name          Name of the systemd service and timer."
    echo "  -c, --command       Command that the service will execute."
    echo "  -f, --frequency     Frequency of the timer (e.g., daily, weekly, *-*-* 00:00:00). Optional if --timer-options is used."
    echo "  -o, --operation     Operation to perform: create (default) or delete."
    echo "  -t, --timer-options Additional systemd timer options (e.g., 'OnBootSec=10min'). Optional."
    exit 1
}

# Parse command-line options
while [[ "$#" -gt 0 ]]; do
    case $1 in
        -n|--name) NAME="$2"; shift ;;
        -c|--command) COMMAND="$2"; shift ;;
        -f|--frequency) FREQUENCY="$2"; shift ;;
        -o|--operation) OPERATION="$2"; shift ;;
        -t|--timer-options) TIMER_OPTIONS="$2"; shift ;;
        *) usage ;;
    esac
    shift
done

# Check if name is provided
if [ -z "$NAME" ]; then
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

        # Prepare frequency or use custom timer options
        TIMER_SPEC=""
        if [ -n "$FREQUENCY" ]; then
            TIMER_SPEC="OnCalendar=$FREQUENCY"
        fi
        if [ -n "$TIMER_OPTIONS" ]; then
            TIMER_SPEC="$TIMER_OPTIONS"
        fi

        # Create the timer file with custom options if provided
        echo "[Unit]
Description=Timer for $NAME

[Timer]
$TIMER_SPEC
Persistent=true

[Install]
WantedBy=timers.target
" | sudo tee $TIMER_FILE

        # Enable and start the timer
        sudo systemctl daemon-reload
        sudo systemctl enable "${NAME}.timer"
        sudo systemctl start "${NAME}.timer"

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

        echo "Service and timer for $NAME deleted successfully."
        ;;
    
    *)
        echo "Invalid operation: $OPERATION"
        usage
        ;;
esac
