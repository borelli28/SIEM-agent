# CEF Agent Documentation

## Components

### 1. Main (`main.rs`)
- Entry point for the application
- Handles CLI commands
- Manages initial setup and configuration
- Initializes file watcher

### 2. Configuration (`config.rs`)
- Stores agent settings in JSON format
- Manages:
  * Agent ID and API key
  * Host ID and Account ID
  * Watch paths
  * SIEM URL

### 3. File Watcher (`watcher.rs`)
- Monitors specified directories for changes
- Detects file modifications
- Sends files to API client for upload
- Manages heartbeat interval

### 4. API Client (`api.rs`)
- Handles communication with SIEM backend
- Manages file uploads
- Tracks upload states
- Implements retry mechanism for failed uploads
- Sends heartbeat signals

### 5. Supporting Modules
- `cli.rs`: Command-line interface definition
- `error.rs`: Custom error types and handling
- `prompt.rs`: User input handling
- `registration.rs`: Agent registration with SIEM

## Basic Workflow

1. **Startup**:
   - Load existing configuration or run initial setup
   - Register agent with SIEM if new setup
   - Initialize file watcher

2. **Monitoring**:
   - Watch specified paths for .log file changes
   - When change detected, attempt upload
   - Track success/failure of uploads

3. **Health Checks**:
   - Send heartbeat every 5 minutes
   - On successful heartbeat, retry failed uploads

4. **Error Recovery**:
   - Track failed uploads
   - Automatically retry when server is available
   - Maintain upload state during downtime