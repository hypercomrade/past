use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::env;
use regex::Regex;
use serde_json::json;
use clap::{Arg, App, ArgMatches, AppSettings};
use thousands::Separable;
use lazy_static::lazy_static;

lazy_static! {
    static ref NAV_COMMANDS: Vec<&'static str> = vec!["cd ", "ls", "pwd", "dir", "pushd", "popd", "ll", "tree", "exa", "fd", "ranger", "nnn", "lf"];
    static ref FILE_OPS: Vec<&'static str> = vec!["cp ", "mv ", "rm ", "mkdir", "touch", "chmod", "chown", "ln ", "rsync", "tar ", 
                   "gzip", "gunzip", "zip", "unzip", "7z", "rename", "trash", "shred"];
    static ref EDITORS: Vec<&'static str> = vec!["vim ", "nano ", "emacs", "code ", "subl ", "gedit", "pico", "vi", "micro", "kate", 
                  "atom", "neovim", "nano", "ed", "sed ", "awk "];
    static ref VCS: Vec<&'static str> = vec!["git ", "hg ", "svn ", "fossil", "bzr", "cvs", "darcs", "git-lfs", "git-flow"];
    static ref PACKAGE_MANAGERS: Vec<&'static str> = vec!["apt", "yum", "dnf", "pacman", "brew", "pip ", "npm ", "snap", "flatpak", 
                           "zypper", "port", "apk", "dpkg", "rpm", "gem", "cargo", "go ", "dotnet"];
    static ref SYSTEM_MONITORS: Vec<&'static str> = vec!["top", "htop", "ps ", "kill", "df ", "du ", "free", "btop", "glances", "nmon", 
                         "iotop", "iftop", "nethogs", "vmstat", "iostat", "dstat", "sar", "mpstat", "pidstat"];
    static ref NETWORK_COMMANDS: Vec<&'static str> = vec!["ssh ", "scp ", "ping", "curl", "wget", "ifconfig", "ip ", "sftp", "ftp", "telnet", 
                           "netstat", "ss", "traceroute", "tracepath", "mtr", "dig", "nslookup", "nmcli", "iwconfig"];
    static ref DATABASES: Vec<&'static str> = vec!["mysql", "psql", "sqlite3", "mongo", "redis-cli", "sqlcmd", "clickhouse-client", 
                    "influx", "cqlsh", "neo4j", "arangosh", "cockroach sql"];
    static ref CONTAINERS: Vec<&'static str> = vec!["docker ", "podman", "kubectl", "oc ", "ctr", "nerdctl", "lxc", "lxd", "vagrant", 
                     "virsh", "qemu", "lima", "colima"];
    static ref SHELL_BUILTINS: Vec<&'static str> = vec!["export", "source", "alias", "echo", "printf", "read", "set", "unset", "type", 
                         "hash", "history", "fc", "jobs", "bg", "fg", "wait", "times", "trap"];
    
    static ref LANGUAGES: Vec<(&'static str, Vec<&'static str>)> = vec![
        ("Rust", vec![
            "cargo", "rustc", "rustup", "rustfmt", "clippy", 
            "cargo build", "cargo run", "cargo test", "cargo check",
            "cargo clippy", "cargo fmt", "cargo doc", "cargo add",
            "cargo update", "cargo install", "cargo publish",
            "cargo tree", "cargo metadata", "cargo audit",
            "cargo deny", "cargo expand", "cargo vendor"
        ]),
        ("Python", vec!["python", "pip", "py ", "python3", "python2", "pylint", "pyflakes", "mypy", "black", "snakemake"]),
        ("Java", vec!["java ", "javac", "mvn ", "gradle", "ant ", "jbang", "groovy"]),
        ("C/C++", vec!["gcc", "g++", "clang", "^make ","$make", "cmake", "ninja", "gdb", "lldb", "valgrind", "cpp"]),
        ("C#", vec!["dotnet", "mono", "msbuild", "csc"]),
        ("JavaScript", vec!["node ", "npm ", "yarn", "deno", "tsc", "bun"]),
        ("Go", vec![" go ","^go","$go", "gofmt", "golangci-lint"]),
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
    ];
}

fn optimized_levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<_> = a.chars().collect();
    let b_chars: Vec<_> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // End if lengths are very different //
    let length_diff = a_len.abs_diff(b_len);
    if length_diff > 5 {
        return length_diff;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;
        let mut min_in_row = i;

        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr_row[j] = std::cmp::min(
                curr_row[j - 1] + 1,
                std::cmp::min(
                    prev_row[j] + 1,
                    prev_row[j - 1] + cost
                )
            );
            min_in_row = std::cmp::min(min_in_row, curr_row[j]);
        }

        // End if minimum in row exceeds threshold //
        let max_len = std::cmp::max(a_len, b_len);
        let threshold = (max_len as f32 * 0.3).ceil() as usize;
        if min_in_row > threshold {
            return min_in_row;
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

fn find_potential_mistypes(commands: &[String], command_frequency: &HashMap<&String, usize>) -> usize {
    let unique_commands: Vec<&String> = command_frequency.keys().copied().collect();
    let mut mistyped_count = 0;

    for cmd in commands {
        // Skip commands that appear multiple times //
        if command_frequency.get(cmd).copied().unwrap_or(0) > 1 {
            continue;
        }

        let mut is_mistyped = true;
        let cmd_len = cmd.len();

        for other_cmd in &unique_commands {
            if cmd == *other_cmd {
                continue;
            }

            let other_len = other_cmd.len();
            // Skip if lengths are too different (could be adjusted later if this seems too strict) //
            if cmd_len.abs_diff(other_len) > 5 {
                continue;
            }

            let distance = optimized_levenshtein(cmd, other_cmd);
            let max_len = std::cmp::max(cmd_len, other_len);
            let similarity_threshold = (max_len as f32 * 0.3).ceil() as usize;

            if distance <= similarity_threshold {
                is_mistyped = false;
                break;
            }
        }

        if is_mistyped {
            mistyped_count += 1;
        }
    }

    mistyped_count
}

fn get_bash_history() -> Result<String, Box<dyn Error>> {
    let home = env::var("HOME")?;
    
    // Try Zsh history first
    let zsh_history_path = Path::new(&home).join(".zsh_history");
    if let Ok(mut file) = File::open(&zsh_history_path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if !contents.is_empty() {
            return Ok(contents);
        }
    }
    
    // Fall back to Bash history
    let bash_history_path = Path::new(&home).join(".bash_history");
    if let Ok(mut file) = File::open(&bash_history_path) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if !contents.is_empty() {
            return Ok(contents);
        }
    }

    // Final fallback to live history
    match Command::new("bash")
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

fn categorize_command(cmd: &str) -> Vec<String> {
    let cmd_lower = cmd.to_lowercase();
    let mut categories = Vec::new();

    if NAV_COMMANDS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Navigation".to_string());
    }
    if FILE_OPS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("File Ops".to_string());
    }
    if EDITORS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Editors".to_string());
    }
    if VCS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Version Ctrl".to_string());
    }
    if PACKAGE_MANAGERS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Pkg Mgmt".to_string());
    }
    if SYSTEM_MONITORS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Sys Monitor".to_string());
    }
    if NETWORK_COMMANDS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Network".to_string());
    }
    if DATABASES.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Databases".to_string());
    }
    if CONTAINERS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Containers".to_string());
    }
    if SHELL_BUILTINS.iter().any(|&x| cmd_lower.contains(x)) {
        categories.push("Shell Builtins".to_string());
    }

    // Language checks (should be expanded)
    for (lang, keywords) in LANGUAGES.iter() {
        if keywords.iter().any(|&x| cmd_lower.contains(x)) {
            categories.push(format!("Lang: {}", lang));
        }
    }

    if categories.is_empty() {
        categories.push("Other".to_string());
    }
    
    categories
}

fn print_brief_stats(commands: &[String], words: &[String]) {
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();
    
    println!("Commands: {} ({} unique)", commands.len(), unique_commands);
    println!("Keywords: {} ({} unique)", words.len(), unique_words);
    
    let avg_len = commands.iter().map(|c| c.len()).sum::<usize>() as f64 / commands.len() as f64;
    let max_len = commands.iter().map(|c| c.len()).max().unwrap_or(0);
    let min_len = commands.iter().map(|c| c.len()).min().unwrap_or(0);
    
    println!("Command length: avg {:.1}, min {}, max {}", avg_len, min_len, max_len);
}

fn print_detailed_analysis(commands: &[String], words: &[String], category_counts: &HashMap<String, usize>) {
    let total_commands = commands.len();
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let total_words = words.len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();
    
    let cmd_lengths: Vec<usize> = commands.iter().map(|c| c.len()).collect();
    let avg_length = cmd_lengths.iter().sum::<usize>() as f64 / total_commands as f64;
    let max_length = *cmd_lengths.iter().max().unwrap_or(&0);
    let min_length = *cmd_lengths.iter().min().unwrap_or(&0);
    
    // Find potentially mistyped commands //
    let command_frequency: HashMap<&String, usize> = commands.iter().fold(HashMap::new(), |mut acc, cmd| {
        *acc.entry(cmd).or_insert(0) += 1;
        acc
    });
    let mistyped_count = find_potential_mistypes(commands, &command_frequency);
    let mistyped_percentage = (mistyped_count as f64 / total_commands as f64) * 100.0;
    
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
    
    println!("\n\x1b[1mBasic Statistics:\x1b[0m");
    println!("- Total commands: {}", total_commands.separate_with_commas());
    println!("- Unique commands: {} ({:.1}% variety)", 
        unique_commands.separate_with_commas(),
        (unique_commands as f64 / total_commands as f64) * 100.0);
    println!("- Potentially mistyped: {} ({:.1}%)", 
        mistyped_count.separate_with_commas(),
        mistyped_percentage);
    println!("- Total keywords: {}", total_words.separate_with_commas());
    println!("- Unique keywords: {} ({:.1}% variety)",
        unique_words.separate_with_commas(),
        (unique_words as f64 / total_words as f64) * 100.0);
    
    println!("\n\x1b[1mCommand Complexity:\x1b[0m");
    println!("- Average length: {:.1} characters", avg_length);
    println!("- Shortest command: {} chars", min_length);
    println!("- Longest command: {} chars", max_length);
    
    println!("\n\x1b[1mCategory Distribution:\x1b[0m");
    let total_categories: usize = category_counts.values().sum();
    let mut sorted_categories: Vec<_> = category_counts.iter().collect();
    sorted_categories.sort_by(|a, b| b.1.cmp(a.1));
    
    for (category, count) in sorted_categories {
        let percentage = (*count as f64 / total_categories as f64) * 100.0;
        println!("- {:20}: {:>5} ({:>5.1}%)", category, count.separate_with_commas(), percentage);
    }
    
    println!("\n\x1b[1mTop Keywords:\x1b[0m");
    for (i, (word, count)) in top_words.iter().enumerate() {
        println!("{}. {:20} {:>5}x", i+1, word, count.separate_with_commas());
    }
}

fn print_bare_stats(commands: &[String], words: &[String], category_counts: &HashMap<String, usize>) {
    let total_commands = commands.len();
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let total_words = words.len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();

    println!("COMMAND STATISTICS");
    println!("------------------");
    println!("Total commands: {}", total_commands);
    println!("Unique commands: {}", unique_commands);
    println!("Command variety: {:.1}%", (unique_commands as f64 / total_commands as f64) * 100.0);
    println!("Total keywords: {}", total_words);
    println!("Unique keywords: {}", unique_words);
    println!("Keyword variety: {:.1}%", (unique_words as f64 / total_words as f64) * 100.0);
    
    let cmd_lengths: Vec<usize> = commands.iter().map(|c| c.len()).collect();
    let avg_length = cmd_lengths.iter().sum::<usize>() as f64 / total_commands as f64;
    println!("Avg command length: {:.1} chars", avg_length);
    
    println!("\nTOP CATEGORIES:");
    let mut sorted_categories: Vec<_> = category_counts.iter().collect();
    sorted_categories.sort_by(|a, b| b.1.cmp(a.1));
    for (category, count) in sorted_categories.iter().take(5) {
        println!("{}: {}", category, count);
    }
}

fn print_boxed_stats(commands: &[String], words: &[String]) {
    let total_commands = commands.len();
    let unique_commands = commands.iter().collect::<HashSet<_>>().len();
    let total_words = words.len();
    let unique_words = words.iter().collect::<HashSet<_>>().len();

    let stats = vec![
        "╔════════════════════════════════════════════╗".to_string(),
        "║               COMMAND PAST                ║".to_string(),
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

    if matches.is_present("bare") {
        print_bare_stats(commands, words, category_counts);
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

    print_boxed_stats(commands, words);
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("past")
        .version("0.1")
        .author("Mikhail Ukrainetz <mckaylbing@gmail.com>")
        .about("The history analysis command for Unix-like systems")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("Use specific history file")
            .takes_value(true))
        .arg(Arg::with_name("json")
            .short("j")
            .long("json")
            .help("Output in JSON format"))
        .arg(Arg::with_name("bare")
            .short("r")
            .long("bare")
            .help("Plain text output without formatting"))
        .arg(Arg::with_name("brief")
            .short("b")
            .long("brief")
            .help("Show brief summary only"))
        .arg(Arg::with_name("detailed")
            .short("d")
            .long("detailed")
            .help("Show detailed analysis"))
        .arg(Arg::with_name("quiet")
            .short("q")
            .long("quiet")
            .help("Suppress non-essential output"))
        .after_help("EXAMPLES:\n  past         # Default boxed output\n  past -r      # Plain text output\n  past -b      # Brief summary\n  past -d      # Detailed analysis\n  past -f ~/.zsh_history  # Analyze zsh history")
        .get_matches();

    let quiet = matches.is_present("quiet");

    if !quiet && !matches.is_present("brief") {
        eprintln!("Analyzing your command history...");
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
        for category in categorize_command(cmd) {
            *category_counts.entry(category).or_insert(0) += 1;
        }
    }

    print_statistics(&commands, &words, &category_counts, &matches);

    Ok(())
}
