#!/bin/bash

set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (use sudo)"
    exit 1
fi

# Create directory for the binary
mkdir -p /opt/cef-agent

# Copy the binary
cp cef_agent /opt/cef-agent/
chmod +x /opt/cef-agent/cef_agent

# Create systemd service file
cat > /etc/systemd/system/cef-agent.service << EOF
[Unit]
Description=CEF Agent Service
After=network.target

[Service]
Type=simple
User=$SUDO_USER
ExecStart=/opt/cef-agent/cef_agent
Restart=always
RestartSec=10
WorkingDirectory=/opt/cef-agent

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd daemon
systemctl daemon-reload

# Enable the service (but don't start it yet)
systemctl enable cef-agent

echo "CEF Agent has been installed!"
echo "Please run 'sudo /opt/cef-agent/cef_agent' to configure the agent"
echo "After configuration, start the service with: sudo systemctl start cef-agent"