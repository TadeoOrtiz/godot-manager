# Godot Version Manager (godot-mgr)

A lightweight and powerful CLI tool written in Rust to manage and run multiple Godot versions seamlessly.

## ✨ Features

- 🚀 **Multiple Versions:** Easily register and switch between different Godot versions (e.g., 4.1, 4.3-dev, 4.2-mono).
- 🎯 **Default Version:** Set a global default version for quick access.
- 🎨 **Interactive Menu:** If no default is set, a beautiful interactive menu lets you choose which version to run.
- 🌈 **Visual Feedback:** Colored terminal output for better readability.
- ⚡ **Seamless Forwarding:** Pass any argument directly to Godot (e.g., `godot -e` or `godot project.godot`).

## 📥 Installation

### Option 1: Direct from GitHub (Recommended)
Run this command to install it directly without cloning:
```bash
cargo install --git https://github.com/TadeoOrtiz/godot-manager.git
```

### Option 2: From Source
1. Ensure you have [Rust](https://www.rust-lang.org/) installed.
2. Clone this repository.
3. Install the binary:
   ```bash
   cargo install --path .
   ```

## 🛠 Usage

The tool uses the `mgr` (or alias `m`) command for management tasks. Any other command is forwarded to the selected Godot version.

### Managing Versions

- **Add a new version:**
  ```bash
  godot mgr add 4.3 /path/to/godot-executable
  ```
- **List all versions:**
  ```bash
  godot m list
  ```
- **Set a default version:**
  ```bash
  godot m default 4.3
  ```
- **Remove a version:**
  ```bash
  godot m remove 4.1
  ```

### Running Godot

Once configured, use the `godot` command as you would normally use the Godot executable:

- **Open the editor:**
  ```bash
  godot -e
  ```
- **Open a specific project:**
  ```bash
  godot path/to/project.godot
  ```
- **Run with specific flags:**
  ```bash
  godot --display-driver wayland --audio-driver PulseAudio
  ```

*If no default version is set, you will be prompted to select one interactively from your registered versions.*

## ⚙️ Configuration

Configuration is stored in your user's standard config directory:
- **Windows:** `%AppData%\godot-manager\config.toml`
- **Linux:** `~/.config/godot-manager/config.toml`
