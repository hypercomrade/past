use std::fs::File;
use std::path::PathBuf;
use std::env;
use std::process::Command;
use std::io::{self, Write};
use std::fs;
use std::os::unix::fs::PermissionsExt;

use serde::{Serialize, Deserialize};
use std::error::Error;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShellConfig {
    pub shell_type: String,
    pub config_file: String,
}

const CONFIG_FILE: &str = ".pastrc";

impl ShellConfig {
    pub fn new(shell_type: String, config_file: String) -> Self {
        ShellConfig { shell_type, config_file }
    }

    pub fn save(&self) -> io::Result<()> {
        let home = env::var("HOME").expect("HOME environment variable not set");
        let config_path = PathBuf::from(home).join(CONFIG_FILE);
        let config_str = toml::to_string(self).expect("Failed to serialize config");
        fs::write(config_path, config_str)
    }

    pub fn load() -> Option<Self> {
        let home = env::var("HOME").ok()?;
        let config_path = PathBuf::from(home).join(CONFIG_FILE);
        let config_str = fs::read_to_string(config_path).ok()?;
        toml::from_str(&config_str).ok()
    }
}

pub fn detect_available_shells() -> Vec<(String, String)> {
    let mut shells = Vec::new();
    
    // Ordered list of shells to check with their config files
    let common_shells = [
        ("zsh", ".zshrc"),
        ("bash", ".bashrc"),
        ("fish", ".config/fish/config.fish"),
        ("ksh", ".kshrc"),
        ("tcsh", ".tcshrc"),
    ];

    // Special case: Homebrew bash on macOS
    #[cfg(target_os = "macos")]
    {
        if Command::new("/usr/local/bin/bash").arg("--version").output().is_ok() {
            shells.push(("bash".to_string(), ".bashrc".to_string()));
        }
    }

    // Check standard locations
    for (shell, config) in &common_shells {
        // Skip bash if we already found Homebrew bash
        #[cfg(target_os = "macos")]
        if *shell == "bash" && !shells.is_empty() && shells[0].0 == "bash" {
            continue;
        }

        if Command::new(shell).arg("--version").output().is_ok() {
            shells.push((shell.to_string(), config.to_string()));
        }
    }

    // Additional macOS-specific checks
    #[cfg(target_os = "macos")]
    {
        // Check for system bash if we haven't found any yet
        if shells.is_empty() && Command::new("/bin/bash").arg("--version").output().is_ok() {
            shells.push(("bash".to_string(), ".bashrc".to_string()));
        }
    }

    shells
}

pub fn prompt_user_for_shell(shells: &[(String, String)]) -> Option<ShellConfig> {
    if shells.is_empty() {
        return None;
    }

    println!("\nAvailable shells detected:");
    for (i, (shell, _)) in shells.iter().enumerate() {
        println!("{}. {}", i + 1, shell);
    }

    loop {
        print!("Please select a shell (1-{}): ", shells.len());
        io::stdout().flush().ok()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok()?;
        let trimmed = input.trim();
        
        if let Ok(choice) = trimmed.parse::<usize>() {
            if choice > 0 && choice <= shells.len() {
                let (shell, config) = &shells[choice - 1];
                return Some(ShellConfig::new(shell.clone(), config.clone()));
            }
        }
        
        println!("Invalid selection. Please enter a number between 1 and {}.", shells.len());
    }
}

pub fn get_shell_config() -> ShellConfig {
    // First try to load existing config
    if let Some(config) = ShellConfig::load() {
        return config;
    }

    let shells = detect_available_shells();
    
    // Always prompt if we detect multiple shells
    if shells.len() > 1 {
        if let Some(config) = prompt_user_for_shell(&shells) {
            if config.save().is_ok() {
                println!("Configuration saved to ~/{}", CONFIG_FILE);
            }
            return config;
        }
    }
    
    // Only fallback automatically if exactly one shell is found
    if shells.len() == 1 {
        let (shell, config) = shells[0].clone();
        println!("Automatically selected detected shell: {}", shell);
        let config = ShellConfig::new(shell, config);
        if config.save().is_ok() {
            println!("Configuration saved to ~/{}", CONFIG_FILE);
        }
        return config;
    }

    // Final fallback (should only happen if no shells detected)
    #[cfg(target_os = "macos")]
    {
        println!("No shells detected, falling back to zsh (macOS default)");
        ShellConfig::new("zsh".to_string(), ".zshrc".to_string())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        println!("No shells detected, falling back to bash");
        ShellConfig::new("bash".to_string(), ".bashrc".to_string())
    }
}

pub fn get_shell_history() -> Result<String, Box<dyn Error>> {
    let config = get_shell_config();
    let home = env::var("HOME")?;
    
    // First try the shell's specific history file
    let history_path = match config.shell_type.as_str() {
        "bash" => {
            // On macOS, check for bash session history
            #[cfg(target_os = "macos")]
            {
                let session_history = PathBuf::from(&home).join(".bash_sessions");
                if session_history.exists() {
                    println!("Warning: macOS bash session history detected. History may be incomplete.");
                }
            }
            PathBuf::from(&home).join(".bash_history")
        },
        "zsh" => PathBuf::from(&home).join(".zsh_history"),
        "fish" => PathBuf::from(&home).join(".local/share/fish/fish_history"),
        "ksh" => PathBuf::from(&home).join(".sh_history"),
        _ => PathBuf::from(&home).join(".bash_history"), // fallback
    };
    
    // Check and fix permissions on macOS
    #[cfg(target_os = "macos")]
    {
        if let Ok(metadata) = fs::metadata(&history_path) {
            let permissions = metadata.permissions();
            if permissions.mode() & 0o777 != 0o600 {
                fs::set_permissions(&history_path, fs::Permissions::from_mode(0o600))
                    .unwrap_or_else(|_| eprintln!("Warning: Could not set permissions on history file"));
            }
        }
    }

    // Special handling for fish history format
    if config.shell_type == "fish" {
        if let Ok(mut file) = File::open(&history_path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            if !contents.is_empty() {
                // Parse fish history format (cmd: <command>)
                let commands: Vec<String> = contents.lines()
                    .filter_map(|line| {
                        if line.starts_with("- cmd: ") {
                            Some(line[7..].to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                return Ok(commands.join("\n"));
            }
        }
    } else if let Ok(mut file) = File::open(&history_path) {
        // Standard handling for other shells
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if !contents.is_empty() {
            return Ok(contents);
        }
    }
    
    // Fallback to using the shell command with shell-specific history commands
    let history_command = match config.shell_type.as_str() {
        "fish" => "history",
        "ksh" => "history -r; history",
        // On macOS, for zsh we might need to force history load
        "zsh" if cfg!(target_os = "macos") => "fc -R; fc -l 1",
        _ => "history -r; history", // bash/zsh default
    };
    
    match Command::new(&config.shell_type)
        .arg("-i")
        .arg("-c")
        .arg(history_command)
        .output() {
        Ok(output) if output.status.success() => {
            let mut history = String::from_utf8(output.stdout)?;
            
            // Clean up fish history output if needed
            if config.shell_type == "fish" {
                history = history.lines()
                    .filter_map(|line| {
                        if line.starts_with("- cmd: ") {
                            Some(line[7..].to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
            }
            
            Ok(history)
        },
        _ => Err("Could not retrieve shell history".into())
    }
}
