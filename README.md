# 🔮 HocusFocus

**HocusFocus** is a lightweight, system-wide website blocker for macOS written in Rust. It runs in the background as a system daemon (`launchd`) and blocks distracting websites according to custom weekly schedules you define.

Unlike browser extensions, HocusFocus works system-wide across all browsers and terminal tools by dynamically updating the `/etc/hosts` file.

---

## ✨ Features

- 📅 **Flexible Weekly Scheduling:** Block websites during work hours (e.g., Mon–Fri 09:00 to 17:00), study blocks, or bedtime.
- 🌙 **Midnight Crossing Support:** Define schedules that span across midnight (e.g., 22:00 to 06:00) natively.
- ⚡ **Dynamic Reloading:** Change your schedules or blocked list in your configuration file, and HocusFocus updates instantly without needing a restart.
- 🔒 **Safe & Clean:** Automatically backs up your original hosts file before the first modification. Upon exiting or uninstalling, it leaves no trace in `/etc/hosts`.
- 🔋 **Zero CPU/Memory Footprint:** Sleeps quietly in the background, consuming zero resources until a state transition occurs.

---

## 🛠️ Installation

Choose one of the two methods below to install HocusFocus:

### Option A: Using Pre-compiled Binaries (Fastest)

1. Go to the **Releases** page on GitHub and download the binary matching your Mac's CPU architecture:
   - **Apple Silicon (M1/M2/M3/M4/M5):** `hocus-focus-aarch64-apple-darwin`
   - **Intel Mac:** `hocus-focus-x86_64-apple-darwin`
2. Open your terminal, navigate to your Downloads folder, and make the binary executable:
   ```bash
   chmod +x hocus-focus-*
   ```
3. Remove macOS's gatekeeper web quarantine flag (required since the binary is self-compiled):
   ```bash
   xattr -d com.apple.quarantine hocus-focus-*
   ```
4. Rename it to a clean command name:
   ```bash
   mv hocus-focus-* hocus-focus
   ```
5. Install it as a macOS system service:
   ```bash
   sudo ./hocus-focus install
   ```

---

### Option B: Building from Source (Requires Rust)

1. Clone this repository and navigate into the directory:
   ```bash
   git clone https://github.com/yourusername/hocus-focus.git
   cd hocus-focus
   ```
2. Build the release binary:
   ```bash
   cargo build --release
   ```
3. Install the service:
   ```bash
   sudo ./target/release/hocus-focus install
   ```

---

## ⚙️ Configuration

HocusFocus loads its configuration file from `~/.config/hocus-focus/config.toml`. 

If it doesn't exist, it will be automatically created on the first install. You can also generate the default file manually by running:
```bash
hocus-focus init
```

### Configuration Example

Open `~/.config/hocus-focus/config.toml` in your favorite text editor:

```toml
# Configuration for Hocus Focus Website Blocker

[[rules]]
name = "Work Focus hours"
domains = [
    "facebook.com",
    "www.facebook.com",
    "instagram.com",
    "www.instagram.com",
    "x.com",
    "www.x.com",
    "reddit.com",
    "www.reddit.com"
]

[[rules.schedules]]
days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
start = "09:00"
end = "12:00"

[[rules.schedules]]
days = ["Mon", "Tue", "Wed", "Thu", "Fri"]
start = "13:00"
end = "17:00"

[[rules]]
name = "Bedtime Block"
domains = [
    "youtube.com",
    "www.youtube.com",
    "netflix.com",
    "www.netflix.com"
]

[[rules.schedules]]
days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
start = "22:30"
end = "06:00" # Overnight schedule supported!
```

> 💡 **Tip:** Since `/etc/hosts` does not support wildcards (e.g. `*.reddit.com`), make sure to list both the bare domain (e.g., `reddit.com`) and its `www` subdomain (e.g., `www.reddit.com`).

---

## 🌐 IMPORTANT: Browser Secure DNS (DNS-over-HTTPS)

Modern web browsers (like Chrome, Brave, Firefox, Zen, and Edge) contain a feature called **Secure DNS** or **DNS-over-HTTPS (DoH)**.

When this is enabled, the browser ignores your computer's OS network settings (including the `/etc/hosts` file) and sends DNS lookups securely over HTTPS directly to DNS servers (like Cloudflare or Google). **This will bypass the HocusFocus blocker.**

To ensure the blocker works, you **must disable Secure DNS** in your browser settings:

- **Brave / Chrome / Edge:** Go to `Settings` -> `Privacy and Security` -> `Security` -> Turn off **Use Secure DNS** (or select "Use current service provider").
- **Firefox / Zen Browser:** Go to `Settings` -> `Privacy & Security` -> Scroll to the bottom to `DNS over HTTPS` -> Select **Max Protection Off** (or set to Default Protection).

---

## 💻 CLI Usage

Once installed, you can use the `hocus-focus` command to manage the daemon:

| Command | Description | Privileges |
| :--- | :--- | :--- |
| `hocus-focus status` | View the background service status, configuration path, and active blocks | User / Admin |
| `hocus-focus check` | Verify what websites are currently blocked based on the current time | User |
| `hocus-focus validate` | Check your configuration file syntax for errors and display active rules | User |
| `hocus-focus daemon` | Run the monitoring loop in the foreground (useful for debugging) | Admin (`sudo`) |
| `hocus-focus install` | Copy the binary to `/usr/local/bin/` and start the launchd service | Admin (`sudo`) |
| `hocus-focus uninstall` | Unload the launchd service, clean up files, and restore `/etc/hosts` | Admin (`sudo`) |

---

## 🧹 Uninstallation

To completely remove HocusFocus, unload the background daemon, delete the system files, and clean the `/etc/hosts` file, run:
```bash
sudo hocus-focus uninstall
```
*(Or `sudo ./target/release/hocus-focus uninstall` if running from your local source directory).*
