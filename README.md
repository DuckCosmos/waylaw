# Waylaw

A fast and lightweight application launcher for Wayland/Hyprland, built with Rust and GTK4.

## Dependencies
Waylaw requires **GTK4** and **gtk4-layer-shell**. Install them according to your distribution before building or running the binary.

**Arch Linux / CachyOS:**
```bash
sudo pacman -S gtk4 gtk4-layer-shell
```

**Fedora:**
```bash
sudo dnf install gtk4 gtk4-layer-shell
```

**Ubuntu / Debian (and derivatives):**
*(Note: gtk4-layer-shell may not be in the default repositories for older versions. If it fails, you may need to compile it from source or use a PPA).*
```bash
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev
```

---

## Installation

Since AUR registration is temporarily closed, you can install Waylaw using the automated script, by downloading the pre-compiled binary, or by building it from source.

### Option 1: Automated Script (Recommended)
The fastest way to install Waylaw is using our installation script. It automatically fetches the latest version from GitHub Releases and installs it to `~/.local/bin`.

Run the following command in your terminal:
```bash
curl -sL https://raw.githubusercontent.com/DuckCosmos/waylaw/main/install.sh | bash
```

### Option 2: Manual Download (Pre-compiled Binary)
If you prefer to do it manually, or if you are on a distribution where the script doesn't work well:
1. Go to the [Releases](https://github.com/DuckCosmos/waylaw/releases) page.
2. Download the latest `.tar.gz` file for `x86_64`.
3. Open your terminal, navigate to your `Downloads` folder, and run:
```bash
tar -xzf waylaw-*.tar.gz
mkdir -p ~/.local/bin
mv waylaw ~/.local/bin/
chmod +x ~/.local/bin/waylaw
```

### Option 3: Build from Source
If you want to compile the project yourself, make sure you have the dependencies installed (see the Dependencies section above), along with Rust and `base-devel` (or `build-essential` on Ubuntu).

1. Clone the repository and enter the directory:
```bash
git clone https://github.com/DuckCosmos/waylaw.git
cd waylaw
```

2. Compile the release binary:
```bash
cargo build --release
```

3. Install the binary to your local path:
```bash
mkdir -p ~/.local/bin
cp target/release/waylaw ~/.local/bin/
chmod +x ~/.local/bin/waylaw
```

---

## Path Configuration (Important)
After installing, ensure that `~/.local/bin` is in your system's `PATH` so you can run the `waylaw` command from anywhere. 

If you get a `command not found` error, add this line to your `~/.bashrc` (or `~/.zshrc`):
```bash
export PATH="$HOME/.local/bin:$PATH"
```
Then, restart your terminal or run `source ~/.bashrc`.

## License
This project is licensed under the GPL-3.0 License. See the [LICENSE](LICENSE) file for details.
