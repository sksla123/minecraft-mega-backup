# mcmgb (Minecraft-MEGA Backup Daemon)

`mcmgb` is a robust, Rust-based daemon designed to safely and automatically back up your Minecraft server to MEGA cloud storage. It ensures data integrity by communicating directly with your Minecraft server via RCON, preventing chunk corruption during the archiving process.

## ✨ Features

* **Data Corruption Prevention:** Integrates natively with Minecraft's RCON protocol to execute `save-off` and `save-all` before archiving, and `save-on` afterward.
* **Automated Retention Policy:** Keeps only the specified number of recent backups, automatically rotating out older versions.
* **Orphaned Backup Cleanup:** Uses a `SUCCESS` file marker. If a backup is interrupted and the marker is missing, the daemon automatically cleans up the corrupted folder on the next run.
* **Smart Storage Check:** Calculates the local Minecraft folder size and verifies if the MEGA account has enough free space (Local Size + Reserve Margin) before starting the upload.
* **Native MEGAcmd Integration:** Uses official MEGAcmd terminal tools for reliable Linux compatibility.
* **Shoutrrr Notifications:** Sends real-time alerts to Discord (or other supported platforms) upon success, failure, or during startup retries.
* **Startup Resilience:** Features configurable retry logic to wait for the Minecraft server (RCON) and MEGA network to become available during a system reboot.

---

## 📋 Prerequisites

Before running `mcmgb`, ensure the following dependencies are installed and configured on your host machine:

1. **Rust Toolchain:** To compile the binary.
2. **MEGAcmd:** The official MEGA command-line tools.
* Must be installed and in your system `$PATH`.
* You must be logged in: `mega-login <email> <password>`


3. **Minecraft Server with RCON Enabled:**
* Ensure `enable-rcon=true` is set in your `server.properties`.



---

## 🚀 Installation & Build

Clone or create the project directory, then build the executable using Cargo:

```bash
# 1. Build the release binary
cargo build --release

# 2. Move the binary to your local bin directory
mkdir -p ~/.local/bin
cp target/release/mcmgb ~/.local/bin/

# 3. Grant execution permissions
chmod +x ~/.local/bin/mcmgb

```

---

## ⚙️ Configuration

Create a `config.toml` file (e.g., in `~/.config/mc-backup/config.toml`). Adjust the values to match your environment.

```toml
[daemon]
# Backup interval (supports 'd' for days, 'h' for hours, 'm' for minutes)
interval = "12h"
# Number of retries during startup (useful if RCON is booting up)
startup_retry = 10
# Interval between startup retries
startup_retry_interval = "1m"

[minecraft]
# Path to your live Minecraft server
server_dir = "/home/minecraft/server"
# Temporary local directory for archiving before upload
backup_temp_dir = "/home/minecraft/backups"

[mega]
# Remote directory in your MEGA account (must use absolute path starting with '/')
remote_dir = "/MinecraftBackups"
# Minimum extra space required in MEGA (in GB) on top of the local server size
reserve_gb = 5
# Number of successful backup versions to keep
keep_versions = 3

[shoutrrr]
# Notification webhook URL (e.g., Discord)
url = "discord://YOUR_TOKEN@YOUR_WEBHOOK_ID"

[rcon]
# Enable native RCON communication for safe backups
enable = true
address = "127.0.0.1:25575"
password = "your_rcon_password"

```

---

## 💻 Usage

### 1. System Check Mode

Before starting the daemon, you can run a comprehensive system check to verify dependencies, directory paths, MEGA login status, RCON connection, and Shoutrrr notifications.

```bash
mcmgb --config ~/.config/mc-backup/config.toml --check

```

### 2. Running the Daemon

To start the daemon in the foreground:

```bash
mcmgb --config ~/.config/mc-backup/config.toml

```

### 3. CLI Arguments

CLI arguments will override the retry settings defined in your `config.toml`.

```bash
mcmgb --help

```

* `-c, --config <CONFIG>`: Path to the configuration file (default: `config.toml`).
* `--check`: Run the system integration check and exit.
* `--retry <RETRY>`: Override startup retry count.
* `--retry-interval <RETRY_INTERVAL>`: Override startup retry interval (e.g., `30s`, `1m`).

---

## 🛠 Systemd Service (Running in Background)

To ensure `mcmgb` runs automatically on system boot, create a systemd service.

1. Create the service file:
```bash
sudo nano /etc/systemd/system/mcmgb.service

```


2. Add the following configuration (replace `/home/clouduser/...` with your actual paths):
```ini
[Unit]
Description=Minecraft MEGA Backup Daemon (mcmgb)
After=network-online.target docker.service

[Service]
Type=simple
User=clouduser
ExecStart=/home/clouduser/.local/bin/mcmgb --config /home/clouduser/Workspace/mcmgb/config.toml
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target

```


3. Enable and start the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now mcmgb.service

```


4. Monitor the logs:
```bash
journalctl -u mcmgb.service -f

```
