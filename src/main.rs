use std::collections::HashMap;
use std::process::Command;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::env;
use std::error::Error;
use regex::Regex;
use std::collections::HashSet;
use serde_json::{json, Value};
use clap::{Arg, App};
use thousands::Separable;

fn get_bash_history() -> Result<String, Box<dyn Error>> {
    // Try reading directly from history file first
    let home = env::var("HOME")?;
    let history_path = Path::new(&home).join(".bash_history");
    
    // First try reading directly from the history file
    if let Ok(mut file) = File::open(&history_path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if !contents.is_empty() {
            return Ok(contents);
        }
    }

    // Fall back to trying the history command
    match Command::new("bash")
        .arg("-i")
        .arg("-c")
        .arg("history -r; history")
        .output() {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8(output.stdout)?)
        },
        _ => {
            // Final fallback - try reading .bash_history again
            let mut file = File::open(&history_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
        }
    }
}

fn process_bash_history(history_text: &str) -> (Vec<String>, Vec<String>) {
    if history_text.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let lines: Vec<&str> = history_text.lines().filter(|line| !line.trim().is_empty()).collect();
    let mut commands = Vec::new();
    let num_re = Regex::new(r"^\s*\d+\s+").unwrap();
    let comment_re = Regex::new(r"^#\d+").unwrap();

    for line in lines {
        let command = if num_re.is_match(line) {
            // Format with line numbers: "  123  command"
            num_re.replace(line, "").into_owned()
        } else if comment_re.is_match(line) {
            // Skip timestamp lines: "#1645483932"
            continue;
        } else if line.starts_with(':') && line.contains(';') {
            // Format from .bash_history: ": 1645483932:0;command"
            match line.splitn(3, ';').nth(2) {
                Some(cmd) => cmd.to_string(),
                None => continue,
            }
        } else {
            // Plain command without any prefixes
            line.to_string()
        };
        commands.push(command.trim().to_string());
    }

    let mut words = Vec::new();
    let token_re = Regex::new(r#"(?:[^\s,"]|"(?:\\.|[^"])*")+"#).unwrap();

    for cmd in &commands {
        for token in token_re.find_iter(cmd) {
            let token = token.as_str();
            if token.starts_with('-') {
                continue;
            }
            let cleaned_token = token.trim_matches(|c| c == '"' || c == '\'').to_lowercase();
            if !cleaned_token.is_empty() && !cleaned_token.chars().all(|c| c.is_ascii_digit()) {
                words.push(cleaned_token);
            }
        }
    }

    (commands, words)
}

fn categorize_command(cmd: &str) -> String {
    let cmd_lower = cmd.to_lowercase();

    // Navigation commands
    let nav_commands = ["cd ", "ls", "pwd", "dir", "pushd", "popd", "ll", "tree", "exa", "fd", "ranger", "nnn", "lf"];
    if nav_commands.iter().any(|&x| cmd_lower.contains(x)) {
        return "Navigation".to_string();
    }

    // File operations
    let file_ops = ["cp ", "mv ", "rm ", "mkdir", "touch", "chmod", "chown", "ln ", "rsync", "tar ", 
                   "gzip", "gunzip", "zip", "unzip", "7z", "rename", "trash", "shred"];
    if file_ops.iter().any(|&x| cmd_lower.contains(x)) {
        return "File Ops".to_string();
    }

    // Editors
    let editors = ["vim ", "nano ", "emacs", "code ", "subl ", "gedit", "pico", "vi", "micro", "kate", 
                  "atom", "neovim", "nano", "ed", "sed ", "awk "];
    if editors.iter().any(|&x| cmd_lower.contains(x)) {
        return "Editors".to_string();
    }

    // Version control
    let vcs = ["git ", "hg ", "svn ", "fossil", "bzr", "cvs", "darcs", "git-lfs", "git-flow"];
    if vcs.iter().any(|&x| cmd_lower.contains(x)) {
        return "Version Ctrl".to_string();
    }

    // Package management
    let package_managers = ["apt", "yum", "dnf", "pacman", "brew", "pip ", "npm ", "snap", "flatpak", 
                           "zypper", "port", "apk", "dpkg", "rpm", "gem", "cargo", "go ", "dotnet"];
    if package_managers.iter().any(|&x| cmd_lower.contains(x)) {
        return "Pkg Mgmt".to_string();
    }

    // System monitoring
    let system_monitors = ["top", "htop", "ps ", "kill", "df ", "du ", "free", "btop", "glances", "nmon", 
                         "iotop", "iftop", "nethogs", "vmstat", "iostat", "dstat", "sar", "mpstat", "pidstat"];
    if system_monitors.iter().any(|&x| cmd_lower.contains(x)) {
        return "Sys Monitor".to_string();
    }

    // Network
    let network_commands = ["ssh ", "scp ", "ping", "curl", "wget", "ifconfig", "ip ", "sftp", "ftp", "telnet", 
                           "netstat", "ss", "traceroute", "tracepath", "mtr", "dig", "nslookup", "nmcli", "iwconfig"];
    if network_commands.iter().any(|&x| cmd_lower.contains(x)) {
        return "Network".to_string();
    }

    // Programming Languages
    let languages: HashMap<&str, Vec<&str>> = [
        ("Python", vec!["python", "pip", "py ", "python3", "python2", "pylint", "pyflakes", "mypy", "black"]),
        ("Java", vec!["java ", "javac", "mvn ", "gradle", "ant ", "jbang", "groovy"]),
        ("Rust", vec!["rustc", "cargo", "rustup", "rustfmt", "clippy"]),
        ("C/C++", vec!["gcc", "g++", "clang", "make ", "cmake", "ninja", "gdb", "lldb", "valgrind", "cpp"]),
        ("C#", vec!["dotnet", "mono", "msbuild", "csc"]),
        ("JavaScript", vec!["node ", "npm ", "yarn", "deno", "tsc", "bun"]),
        ("Go", vec!["go ", "gofmt", "golangci-lint"]),
        ("Ruby", vec!["ruby ", "gem ", "rake", "bundle"]),
        ("PHP", vec!["php ", "composer", "phpunit"]),
        ("Shell", vec!["bash ", "sh ", "zsh ", "fish ", "dash", "ksh"]),
        ("Assembly", vec!["as ", "nasm", "yasm", "objdump", "gdb"]),
        ("R", vec!["r ", "rscript", "radian"]),
        ("Perl", vec!["perl ", "cpan"]),
        ("Haskell", vec!["ghc", "ghci", "stack", "cabal"]),
        ("Lua", vec!["lua ", "luac"]),
        ("Dart", vec!["dart ", "flutter"]),
        ("Scala", vec!["scala ", "scalac"]),
        ("Kotlin", vec!["kotlin", "kotlinc"]),
        ("Swift", vec!["swift ", "swiftc"]),
    ].iter().cloned().collect();

    for (lang, keywords) in languages {
        if keywords.iter().any(|&x| cmd_lower.contains(x)) {
            return format!("Lang: {}", lang);
        }
    }

    // Databases
    let databases = ["mysql", "psql", "sqlite3", "mongo", "redis-cli", "sqlcmd", "clickhouse-client", 
                    "influx", "cqlsh", "neo4j", "arangosh", "cockroach sql"];
    if databases.iter().any(|&x| cmd_lower.contains(x)) {
        return "Databases".to_string();
    }

    // Containers/Virtualization
    let containers = ["docker ", "podman", "kubectl", "oc ", "ctr", "nerdctl", "lxc", "lxd", "vagrant", 
                     "virsh", "qemu", "lima", "colima"];
    if containers.iter().any(|&x| cmd_lower.contains(x)) {
        return "Containers".to_string();
    }

    // Shell builtins
    let shell_builtins = ["export", "source", "alias", "echo", "printf", "read", "set", "unset", "type", 
                         "hash", "history", "fc", "jobs", "bg", "fg", "wait", "times", "trap"];
    if shell_builtins.iter().any(|&x| cmd_lower.contains(x)) {
        return "Shell Builtins".to_string();
    }

    // If none match
    "Other".to_string()
}

fn print_statistics(commands: &[String], words: &[String], category_counts: &HashMap<String, usize>, output_format: &str) {
    let total_commands = commands.len();
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let total_words = words.len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();

    // Calculate most frequent commands
    let mut command_counts = HashMap::new();
    for cmd in commands {
        *command_counts.entry(cmd.clone()).or_insert(0) += 1;
    }
    let mut command_counts: Vec<_> = command_counts.into_iter().collect();
    command_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let top_commands = command_counts.iter().take(10).collect::<Vec<_>>();

    // Calculate most frequent words
    let mut word_counts = HashMap::new();
    for word in words {
        *word_counts.entry(word.clone()).or_insert(0) += 1;
    }
    let mut word_counts: Vec<_> = word_counts.into_iter().collect();
    word_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let top_words = word_counts.iter().take(10).collect::<Vec<_>>();

    // Calculate category distribution
    let _total_categories: usize = category_counts.values().sum();
    let mut category_counts: Vec<_> = category_counts.iter().collect();
    category_counts.sort_by(|a, b| b.1.cmp(a.1));
    let top_categories = category_counts.iter().take(10).collect::<Vec<_>>();

    if output_format == "json" {
        let result = json!({
            "summary": {
                "total_commands": total_commands,
                "unique_commands": unique_commands,
                "command_variety": unique_commands as f64 / total_commands as f64,
                "total_keywords": total_words,
                "unique_keywords": unique_words,
                "keyword_variety": unique_words as f64 / total_words as f64
            },
            "top_commands": top_commands.iter().map(|(cmd, count)| {
                json!({"command": cmd, "count": count})
            }).collect::<Vec<Value>>(),
            "top_words": top_words.iter().map(|(word, count)| {
                json!({"word": word, "count": count})
            }).collect::<Vec<Value>>(),
            "top_categories": top_categories.iter().map(|(cat, count)| {
                json!({"category": cat, "count": count})
            }).collect::<Vec<Value>>(),
            "all_categories": category_counts.iter().map(|(cat, count)| (cat, count)).collect::<HashMap<_, _>>()
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    // Prepare statistics text with box
    let mut stats = vec![
        "╔════════════════════════════════════════════╗".to_string(),
        "║          COMMAND HISTORY ANALYSIS          ║".to_string(),
        "╟────────────────────────────────────────────╢".to_string(),
        format!("║ {:<20} {:>12} ║", "Total commands:", total_commands.separate_with_commas()),
        format!("║ {:<20} {:>12} ║", "Unique commands:", unique_commands.separate_with_commas()),
        format!("║ {:<20} {:>12.1}% ║", "Command variety:", (unique_commands as f64 / total_commands as f64) * 100.0),
        "╟────────────────────────────────────────────╢".to_string(),
        format!("║ {:<20} {:>12} ║", "Total keywords:", total_words.separate_with_commas()),
        format!("║ {:<20} {:>12} ║", "Unique keywords:", unique_words.separate_with_commas()),
        format!("║ {:<20} {:>12.1}% ║", "Keyword variety:", (unique_words as f64 / total_words as f64) * 100.0),
        "╟────────────────────────────────────────────╢".to_string(),
        "║           MOST FREQUENT COMMANDS           ║".to_string(),
    ];

    // Add top commands
    for (i, (cmd, count)) in top_commands.iter().enumerate() {
        let cmd_text = format!("{}. {:<33}{:>5}", i+1, if cmd.len() > 30 { &cmd[..30] } else { cmd }, count.separate_with_commas());
        stats.push(format!("║ {} ║", cmd_text));
    }

    stats.push("╟────────────────────────────────────────────╢".to_string());
    stats.push("║            MOST FREQUENT WORDS             ║".to_string());

    // Add top words
    for (i, (word, count)) in top_words.iter().enumerate() {
        let word_text = format!("{}. {:<33}{:>5}", i+1, if word.len() > 30 { &word[..30] } else { word }, count.separate_with_commas());
        stats.push(format!("║ {} ║", word_text));
    }

    stats.push("╟────────────────────────────────────────────╢".to_string());
    stats.push("║             TOP CATEGORIES                 ║".to_string());

    // Add top categories
    for (i, (cat, count)) in top_categories.iter().enumerate() {
        let cat_text = format!("{}. {:<33}{:>5}", i+1, if cat.len() > 30 { &cat[..30] } else { cat }, count.separate_with_commas());
        stats.push(format!("║ {} ║", cat_text));
    }

    stats.push("╚════════════════════════════════════════════╝".to_string());

    // Print with colored output if available
    if let Some(mut colored) = term::stdout() {
        colored.fg(term::color::CYAN).unwrap();
        for line in stats {
            println!("{}", line);
        }
        colored.reset().unwrap();
    } else {
        for line in stats {
            println!("{}", line);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Bash History Analyzer")
        .version("1.0")
        .author("Your Name")
        .about("Analyze bash command history with various output options")
        .arg(Arg::with_name("file")
             .short("f")
             .long("file")
             .value_name("FILE")
             .help("Use a specific history file instead of live bash history")
             .takes_value(true))
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("FILE")
             .help("Output file for visualization (PNG, JPG, SVG, PDF)")
             .takes_value(true))
        .arg(Arg::with_name("top-n")
             .short("n")
             .long("top-n")
             .value_name("N")
             .help("Number of top commands/words to display")
             .default_value("15")
             .takes_value(true))
        .arg(Arg::with_name("json")
             .short("j")
             .long("json")
             .help("Output results in JSON format"))
        .arg(Arg::with_name("visualize")
             .short("v")
             .long("visualize")
             .help("Generate visualizations (interactive or to file if --output specified)"))
        .arg(Arg::with_name("quiet")
             .short("q")
             .long("quiet")
             .help("Suppress all non-essential output (except JSON if requested)"))
        .get_matches();

    let quiet = matches.is_present("quiet");

    if !quiet {
        eprintln!("Bash History Analyzer - Loading your command history...");
    }

    let history_text = if let Some(file) = matches.value_of("file") {
        let mut file = File::open(Path::new(file))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    } else {
        match get_bash_history() {
            Ok(text) => {
                if text.trim().is_empty() {
                    eprintln!("Warning: Retrieved empty history. Trying fallback method...");
                    let home = env::var("HOME")?;
                    let mut file = File::open(Path::new(&home).join(".bash_history"))?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    contents
                } else {
                    text
                }
            },
            Err(e) => {
                if !quiet {
                    eprintln!("Failed to get live bash history ({}). Trying fallback method...", e);
                }
                let home = env::var("HOME")?;
                let mut file = File::open(Path::new(&home).join(".bash_history"))?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                contents
            }
        }
    };

    let (commands, words) = process_bash_history(&history_text);

    if commands.is_empty() {
        eprintln!("No valid commands found in the history.");
        std::process::exit(1);
    }

    if !quiet {
        eprintln!("\nAnalyzed {} commands with {} keywords.", commands.len(), words.len());
    }

    // Get categories for statistics
    let mut category_counts = HashMap::new();
    for cmd in &commands {
        let category = categorize_command(cmd);
        *category_counts.entry(category).or_insert(0) += 1;
    }

    // Handle output options
    if matches.is_present("json") {
        print_statistics(&commands, &words, &category_counts, "json");
    } else if !quiet {
        print_statistics(&commands, &words, &category_counts, "text");
    }

    // Handle visualization
    if matches.is_present("visualize") || matches.is_present("output") {
        eprintln!("Visualization functionality is not implemented in this Rust version.");
        eprintln!("Consider using the Python version for visualization features.");
    }

    Ok(())
}