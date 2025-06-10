## past

The history analysis command for Unix-like shells (sorry powershell fans).

The goal of past is so be sort a history+ command. Allowing you to see usage patterns, trends, and generally look back on your
command history in a more modern way.

### **Supported Shells**
![Bash](https://img.shields.io/badge/Shell-Bash-green?logo=gnu-bash)
![Zsh](https://img.shields.io/badge/Shell-Zsh-blue?logo=zsh)

### **Supported Architectures**
![Debian x86_64](https://img.shields.io/badge/Debian-x86__64-red?logo=debian)
![Debian ARM64](https://img.shields.io/badge/Debian-ARM64-red?logo=debian)
![macOS Intel](https://img.shields.io/badge/macOS-x86__64-black?logo=apple)
![macOS ARM](https://img.shields.io/badge/macOS-ARM64-black?logo=apple)

### **Contributing**
If you would like to contribute to this project, download rust (via cargo or brew) clone this repository as shown below

```
git clone https://github.com/KaylBing/past 
```

Then, move into the directory and build youre files
```
cargo build --release
```

Lastly, move into the target release directory, and run the past commmand
```
./past --help
```
