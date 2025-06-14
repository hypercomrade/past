use std::fs::File;
use std::path::PathBuf;
use std::env;
use std::process::Command;
use std::io::{self, Write};
use std::fs;

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
    let common_shells = [
        ("bash", ".bashrc"),
        ("zsh", ".zshrc"),
        ("fish", ".config/fish/config.fish"),
        ("ksh", ".kshrc"),
        ("tcsh", ".tcshrc"),
    ];

    for (shell, config) in &common_shells {
        if Command::new(shell).arg("--version").output().is_ok() {
            shells.push((shell.to_string(), config.to_string()));
        }
    }

    shells
}

pub fn prompt_user_for_shell(shells: &[(String, String)]) -> Option<ShellConfig> {
    if shells.is_empty() {
        return None;
    }

    if shells.len() == 1 {
        let (shell, config) = &shells[0];
        println!("Detected shell: {}", shell);
        return Some(ShellConfig::new(shell.clone(), config.clone()));
    }

    println!("Multiple shells detected. Please select one:");
    for (i, (shell, _)) in shells.iter().enumerate() {
        println!("{}. {}", i + 1, shell);
    }

    print!("Enter your choice (1-{}): ", shells.len());
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let choice: usize = input.trim().parse().ok()?;

    if choice > 0 && choice <= shells.len() {
        let (shell, config) = &shells[choice - 1];
        Some(ShellConfig::new(shell.clone(), config.clone()))
    } else {
        None
    }
}

pub fn get_shell_config() -> ShellConfig {
    if let Some(config) = ShellConfig::load() {
        return config;
    }

    let shells = detect_available_shells();
    if let Some(config) = prompt_user_for_shell(&shells) {
        if config.save().is_ok() {
            println!("Configuration saved to ~/{}", CONFIG_FILE);
        }
        config
    } else {
        // Fallback to bash if nothing else works //
        ShellConfig::new("bash".to_string(), ".bashrc".to_string())
    }
}

pub fn get_shell_history() -> Result<String, Box<dyn Error>> {
    let config = get_shell_config();
    let home = env::var("HOME")?;
    
    // First try the shell's specific history file //
    let history_path = match config.shell_type.as_str() {
        "bash" => PathBuf::from(&home).join(".bash_history"),
        "zsh" => PathBuf::from(&home).join(".zsh_history"),
        "fish" => PathBuf::from(&home).join(".local/share/fish/fish_history"),
        _ => PathBuf::from(&home).join(".bash_history"), // fallback
    };
    
    if let Ok(mut file) = File::open(&history_path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if !contents.is_empty() {
            return Ok(contents);
        }
    }
    
    // Fallback to using the shell command //
    match Command::new(&config.shell_type)
        .arg("-i")
        .arg("-c")
        .arg("history -r; history")
        .output() {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8(output.stdout)?)
        },
        _ => Err("Could not retrieve shell history".into())
    }
}
