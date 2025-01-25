# CEF Agent Installation Guide

## Prerequisites
- Linux operating system
- Root/sudo privileges
- Valid Host ID and Account ID from your SIEM backend

## Installation Steps

1. Make the install script executable:
```
chmod +x install.sh
```

2. Run the installation script:
```
sudo ./install.sh
```

The agent will now:
- Be installed as a system service
- Start automatically on system boot
- Run in the background

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
3. Ensure the agent configuration file exists and is valid