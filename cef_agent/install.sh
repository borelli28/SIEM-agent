#!/bin/bash

# install.sh
set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Get the absolute path of the agent binary
AGENT_PATH=$(readlink -f ./cef_agent)
AGENT_DIR=$(dirname "$AGENT_PATH")

# Create systemd service file
cat > /etc/systemd/system/cef-agent.service << EOF
[Unit]
Description=CEF Agent Service
After=network.target

[Service]
Type=simple
User=$SUDO_USER
ExecStart=$AGENT_PATH
Restart=always
RestartSec=10
WorkingDirectory=$AGENT_DIR

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd daemon
systemctl daemon-reload

# Enable and start the service
systemctl enable cef-agent
systemctl start cef-agent

echo "CEF Agent has been installed and started as a system service!"
echo "You can check its status with: systemctl status cef-agent"