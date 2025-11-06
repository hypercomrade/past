## past

**The history analysis command for Unix-like shells** (sorry PowerShell fans).

The goal of `past` is to be modern take on the `history` command - allowing you to see usage patterns, trends, and generally look back on your command history in a feature rich way. Currently features include:

- Summary statistics of your command usage (`--brief or --detailed`)
- Category-based search (`-C/--category`)
- Keyword search (`-s/--search`)
- Interactive search mode (`-i/--interactive`)

### **Officially Supported Shells**
![Bash](https://img.shields.io/badge/Shell-Bash-green?logo=gnu-bash)
![Zsh](https://img.shields.io/badge/Shell-Zsh-blue?logo=zsh)
![Fish](https://img.shields.io/badge/Shell-Fish-yellow?logo=fish)
![Ksh](https://img.shields.io/badge/Shell-Ksh-lightgrey?logo=terminal)

### **Officially Supported Systems**
![Debian x86_64](https://img.shields.io/badge/Debian-x86__64-red?logo=debian)
![Debian ARM64](https://img.shields.io/badge/Debian-ARM64-red?logo=debian)
![macOS Intel](https://img.shields.io/badge/macOS-x86__64-black?logo=apple)
![macOS ARM](https://img.shields.io/badge/macOS-ARM64-black?logo=apple)

Please keep in mind that past can still run on many non listed systems. These are just the most supported and documented ones.

### **Quick Start**
```bash
# Clone and build
https://github.com/hypercomrade/past
cd past
cargo build --release

# Aliasing (until we are approved for brew and apt)
# The cleanest way to use our command for now, is to add something like this to your .bashrc

alias past='/path/to/repo/past/target/release/past'
