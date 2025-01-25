#!/bin/bash

# uninstall.sh
set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Stop and disable the service
systemctl stop cef-agent
systemctl disable cef-agent

# Remove the service file
rm -f /etc/systemd/system/cef-agent.service

# Reload systemd daemon
systemctl daemon-reload

echo "CEF Agent service has been uninstalled!"