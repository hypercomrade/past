use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::env;
use std::error::Error;
use regex::Regex;
use serde_json::{json, Value};
use clap::{Arg, App, ArgMatches};
use thousands::Separable;

fn get_bash_history() -> Result<String, Box<dyn Error>> {
    let home = env::var("HOME")?;
    let history_path = Path::new(&home).join(".bash_history");
    
    if let Ok(mut file) = File::open(&history_path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if !contents.is_empty() {
            return Ok(contents);
        }
    }

    match Command::new("bash")
        .arg("-i")
        .arg("-c")
        .arg("history -r; history")
        .output() {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8(output.stdout)?)
        },
        _ => {
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
            num_re.replace(line, "").into_owned()
        } else if comment_re.is_match(line) {
            continue;
        } else if line.starts_with(':') && line.contains(';') {
            match line.splitn(3, ';').nth(2) {
                Some(cmd) => cmd.to_string(),
                None => continue,
            }
        } else {
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
    let nav_commands = ["cd ", "ls", "pwd", "dir", "pushd", "popd", "ll", "tree", "exa", "fd", "ranger", "nnn", "lf"];
    let file_ops = ["cp ", "mv ", "rm ", "mkdir", "touch", "chmod", "chown", "ln ", "rsync", "tar ", 
                   "gzip", "gunzip", "zip", "unzip", "7z", "rename", "trash", "shred"];
    let editors = ["vim ", "nano ", "emacs", "code ", "subl ", "gedit", "pico", "vi", "micro", "kate", 
                  "atom", "neovim", "nano", "ed", "sed ", "awk "];
    let vcs = ["git ", "hg ", "svn ", "fossil", "bzr", "cvs", "darcs", "git-lfs", "git-flow"];
    let package_managers = ["apt", "yum", "dnf", "pacman", "brew", "pip ", "npm ", "snap", "flatpak", 
                           "zypper", "port", "apk", "dpkg", "rpm", "gem", "cargo", "go ", "dotnet"];
    let system_monitors = ["top", "htop", "ps ", "kill", "df ", "du ", "free", "btop", "glances", "nmon", 
                         "iotop", "iftop", "nethogs", "vmstat", "iostat", "dstat", "sar", "mpstat", "pidstat"];
    let network_commands = ["ssh ", "scp ", "ping", "curl", "wget", "ifconfig", "ip ", "sftp", "ftp", "telnet", 
                           "netstat", "ss", "traceroute", "tracepath", "mtr", "dig", "nslookup", "nmcli", "iwconfig"];
    let databases = ["mysql", "psql", "sqlite3", "mongo", "redis-cli", "sqlcmd", "clickhouse-client", 
                    "influx", "cqlsh", "neo4j", "arangosh", "cockroach sql"];
    let containers = ["docker ", "podman", "kubectl", "oc ", "ctr", "nerdctl", "lxc", "lxd", "vagrant", 
                     "virsh", "qemu", "lima", "colima"];
    let shell_builtins = ["export", "source", "alias", "echo", "printf", "read", "set", "unset", "type", 
                         "hash", "history", "fc", "jobs", "bg", "fg", "wait", "times", "trap"];

    if nav_commands.iter().any(|&x| cmd_lower.contains(x)) { return "Navigation".to_string(); }
    if file_ops.iter().any(|&x| cmd_lower.contains(x)) { return "File Ops".to_string(); }
    if editors.iter().any(|&x| cmd_lower.contains(x)) { return "Editors".to_string(); }
    if vcs.iter().any(|&x| cmd_lower.contains(x)) { return "Version Ctrl".to_string(); }
    if package_managers.iter().any(|&x| cmd_lower.contains(x)) { return "Pkg Mgmt".to_string(); }
    if system_monitors.iter().any(|&x| cmd_lower.contains(x)) { return "Sys Monitor".to_string(); }
    if network_commands.iter().any(|&x| cmd_lower.contains(x)) { return "Network".to_string(); }
    if databases.iter().any(|&x| cmd_lower.contains(x)) { return "Databases".to_string(); }
    if containers.iter().any(|&x| cmd_lower.contains(x)) { return "Containers".to_string(); }
    if shell_builtins.iter().any(|&x| cmd_lower.contains(x)) { return "Shell Builtins".to_string(); }

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

    "Other".to_string()
}

fn print_brief_stats(commands: &[String], words: &[String]) {
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();
    
    println!("Commands: {} ({} unique)", commands.len(), unique_commands);
    println!("Keywords: {} ({} unique)", words.len(), unique_words);
    
    // Calculate command length statistics
    let avg_len = commands.iter().map(|c| c.len()).sum::<usize>() as f64 / commands.len() as f64;
    let max_len = commands.iter().map(|c| c.len()).max().unwrap_or(0);
    let min_len = commands.iter().map(|c| c.len()).min().unwrap_or(0);
    
    println!("Command length: avg {:.1}, min {}, max {}", avg_len, min_len, max_len);
}

fn print_detailed_analysis(commands: &[String], words: &[String], category_counts: &HashMap<String, usize>) {
    // Calculate basic statistics
    let total_commands = commands.len();
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let total_words = words.len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();
    
    // Command complexity analysis
    let cmd_lengths: Vec<usize> = commands.iter().map(|c| c.len()).collect();
    let avg_length = cmd_lengths.iter().sum::<usize>() as f64 / total_commands as f64;
    let max_length = *cmd_lengths.iter().max().unwrap_or(&0);
    let min_length = *cmd_lengths.iter().min().unwrap_or(&0);
    
    // Word frequency analysis
    let mut word_counts = HashMap::new();
    for word in words {
        *word_counts.entry(word.clone()).or_insert(0) += 1;
    }
    let top_words: Vec<_> = {
        let mut v: Vec<_> = word_counts.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.into_iter().take(5).collect()
    };
    
    println!("\n\x1b[1;34m=== DETAILED ANALYSIS ===\x1b[0m");
    
    // Basic statistics
    println!("\n\x1b[1mBasic Statistics:\x1b[0m");
    println!("- Total commands: {}", total_commands.separate_with_commas());
    println!("- Unique commands: {} ({:.1}% variety)", 
        unique_commands.separate_with_commas(),
        (unique_commands as f64 / total_commands as f64) * 100.0);
    println!("- Total keywords: {}", total_words.separate_with_commas());
    println!("- Unique keywords: {} ({:.1}% variety)",
        unique_words.separate_with_commas(),
        (unique_words as f64 / total_words as f64) * 100.0);
    
    // Command complexity
    println!("\n\x1b[1mCommand Complexity:\x1b[0m");
    println!("- Average length: {:.1} characters", avg_length);
    println!("- Shortest command: {} chars", min_length);
    println!("- Longest command: {} chars", max_length);
    
    // Category breakdown
    println!("\n\x1b[1mCategory Distribution:\x1b[0m");
    let total_categories: usize = category_counts.values().sum();
    let mut sorted_categories: Vec<_> = category_counts.iter().collect();
    sorted_categories.sort_by(|a, b| b.1.cmp(a.1));
    
    for (category, count) in sorted_categories {
        let percentage = (*count as f64 / total_categories as f64) * 100.0;
        println!("- {:20}: {:>5} ({:>5.1}%)", category, count.separate_with_commas(), percentage);
    }
    
    // Top words
    println!("\n\x1b[1mTop Keywords:\x1b[0m");
    for (i, (word, count)) in top_words.iter().enumerate() {
        println!("{}. {:20} {:>5}x", i+1, word, count.separate_with_commas());
    }
}

fn print_statistics(commands: &[String], words: &[String], category_counts: &HashMap<String, usize>, matches: &ArgMatches) {
    if matches.is_present("json") {
        let result = json!({
            "commands": commands.len(),
            "unique_commands": commands.iter().collect::<HashSet<_>>().len(),
            "words": words.len(),
            "unique_words": words.iter().collect::<HashSet<_>>().len(),
            "categories": category_counts
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    if matches.is_present("brief") {
        print_brief_stats(commands, words);
        return;
    }

    if matches.is_present("detailed") {
        print_detailed_analysis(commands, words, category_counts);
        return;
    }

    // Default output (similar to original box output)
    let total_commands = commands.len();
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let total_words = words.len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();

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
        "╚════════════════════════════════════════════╝".to_string(),
    ];

    for line in stats {
        println!("{}", line);
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
        .arg(Arg::with_name("json")
             .short("j")
             .long("json")
             .help("Output results in JSON format"))
        .arg(Arg::with_name("brief")
             .long("brief")
             .help("Show only minimal summary output"))
        .arg(Arg::with_name("detailed")
             .long("detailed")
             .help("Show extended detailed analysis"))
        .arg(Arg::with_name("quiet")
             .short("q")
             .long("quiet")
             .help("Suppress all non-essential output (except JSON if requested)"))
        .get_matches();

    let quiet = matches.is_present("quiet");

    if !quiet && !matches.is_present("brief") {
        eprintln!("Bash History Analyzer - Loading your command history...");
    }

    let history_text = if let Some(file) = matches.value_of("file") {
        let mut file = File::open(Path::new(file))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    } else {
        match get_bash_history() {
            Ok(text) => text,
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

    let mut category_counts = HashMap::new();
    for cmd in &commands {
        let category = categorize_command(cmd);
        *category_counts.entry(category).or_insert(0) += 1;
    }

    print_statistics(&commands, &words, &category_counts, &matches);

    Ok(())
}