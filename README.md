# minipp

Quickly identify unused files in your project to help slim down your codebase.

A cross-platform command-line tool for Windows, macOS (x86_64/arm64), and Linux (x86_64/arm64).

It's `Rust version`.

You can see nodejs version at [https://github.com/laoer536/minipp](https://github.com/laoer536/minipp)

## 🤩 Performance：

For such a front-end project, there are `9843` files under the src folder:

![image.png](https://s2.loli.net/2025/06/21/WzyNGFKScLdBP62.png)

### Rust-minipp 269ms

![image.png](https://s2.loli.net/2025/06/21/3rFtOCib2nagqvS.png)

### Nodejs-minipp 2.851s

![image.png](https://s2.loli.net/2025/06/21/3xcpfM2mVKREIUY.png)

> **Rust has a performance improvement of about 10 times compared to Nodejs! 🤪**

## 🚀 Installation

### Method 1: Download Prebuilt Binaries

1. Go to the [Releases page](https://github.com/laoer536/minipp-rs/releases) and download the archive for your system (
   `minipp-linux-x86_64.tar.gz`, `minipp-macos-arm64.tar.gz`, `minipp-windows-x86_64.zip`, etc.).
2. Extract the archive, and you will get the `minipp` binary (or `minipp.exe` on Windows).

#### On macOS / Linux

```sh
# Make the binary executable
chmod +x minipp

# Move it to a directory in your PATH, e.g., ~/.local/bin or /usr/local/bin
mv minipp ~/.local/bin/

# If ~/.local/bin is not in your PATH, add it:
echo 'export PATH=$HOME/.local/bin:$PATH' >> ~/.bashrc
source ~/.bashrc

# Test the installation (Execute in the root directory of your TS project or React+TS project.)
minipp
```

#### On Windows

1. Copy `minipp.exe` to a directory that is included in your system PATH (e.g., `C:\Windows\System32` or
   `C:\Users\YourName\.cargo\bin`).
2. Open a new Command Prompt or PowerShell window and type (Execute in the root directory of your TS project or React+TS
   project.) :

```bat
minipp
```

---

### Method 2: Install via cargo-binstall (if published to crates.io)

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) installed:

```sh
cargo binstall minipp
```

---

## 🛠 Usage

Execute in the root directory of your TS project or React+TS project.

Simply run in your terminal:

```sh
minipp
```

---

## ❓ FAQ

- If you get a "command not found" or "'minipp' is not recognized..." error, make sure the binary's directory is in your
  `PATH`.
- On macOS/Linux, if you get a permission denied error, run: `chmod +x minipp`

---

## ✨ Feedback & Contributions

Feel free to open issues or pull requests in the [Issues](https://github.com/laoer536/minipp-rs/issues) page.

---

> This project supports Windows, macOS (x86_64/arm64), and Linux (x86_64/arm64). Thank you for using `minipp`!