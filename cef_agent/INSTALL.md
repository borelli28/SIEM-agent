# CEF Agent Installation Guide

## Prerequisites
- Linux operating system
- Root/sudo privileges
- Valid Host ID and Account ID from your SIEM settings

## Installation Steps

1. Make the install script executable:
```
chmod +x install.sh
```

2. Run the installation script:
```
sudo ./install.sh
```

3. Configure the agent (first-time setup required):
```
sudo /opt/cef-agent/cef_agent
```
You will be prompted for:
- Host ID (from your SIEM settings)
- Account ID (from your SIEM settings)
- Hostname
- SIEM URL
- Paths to monitor

4. After configuration, start the service:
```
sudo systemctl start cef-agent
```

The agent will now:
- Run as a system service
- Start automatically on system boot
- Monitor configured paths

## Verify Installation

Check if the service is running:
```
systemctl status cef-agent
```

## Service Management

### Basic Commands
- Stop the agent: `sudo systemctl stop cef-agent`
- Start the agent: `sudo systemctl start cef-agent`
- Restart the agent: `sudo systemctl restart cef-agent`
- View logs: `journalctl -u cef-agent`

### Configuration Commands
- Add a path to monitor: `sudo /opt/cef-agent/cef_agent config add-path /path/to/watch`
- Remove a path: `sudo /opt/cef-agent/cef_agent config remove-path /path/to/remove`
- List monitored paths: `sudo /opt/cef-agent/cef_agent config list-paths`
- Set SIEM URL: `sudo /opt/cef-agent/cef_agent config set-url http://your-siem-url`

Note: After changing configuration, restart the service:
```
sudo systemctl restart cef-agent
```

### Service Status
The agent service will:
- Start automatically when the system boots
- Restart automatically if it crashes
- Log its activity to system journal

## Uninstallation

1. Make the uninstall script executable:
```
chmod +x uninstall.sh
```

2. Run the uninstallation script:
```
sudo ./uninstall.sh
```

## Troubleshooting

If you encounter issues:
1. Check the service status: `systemctl status cef-agent`
2. View the logs: `journalctl -u cef-agent -f`
3. Ensure the agent configuration is valid
4. Make sure your SIEM backend is accessible
5. Verify your Host ID and Account ID are correct