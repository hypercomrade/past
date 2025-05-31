import re
import subprocess
from collections import Counter
import matplotlib.pyplot as plt
import textwrap

def get_bash_history():
    """Get bash history by running the history command"""
    try:
        result = subprocess.run(['bash', '-i', '-c', 'history'], 
                               capture_output=True, text=True, timeout=10)
        if result.returncode == 0:
            return result.stdout
        else:
            print("Error running history command:", result.stderr)
            return None
    except Exception as e:
        print(f"Error getting bash history: {e}")
        return None

def process_bash_history(history_text):
    """Process bash history and extract commands and keywords"""
    if not history_text:
        return [], []
        
    lines = [line.strip() for line in history_text.split('\n') if line.strip()]
    
    commands = []
    for line in lines:
        if re.match(r'^\s*\d+\s+', line):
            command = re.sub(r'^\s*\d+\s+', '', line)
        elif re.match(r'^#\d+', line):
            continue
        else:
            command = line
        commands.append(command)
    
    words = []
    for cmd in commands:
        tokens = re.findall(r'(?:[^\s,"]|"(?:\\.|[^"])*")+', cmd)
        for token in tokens:
            if token.startswith('-'):
                continue
            token = token.strip('"\'').lower()
            if token and not token.isdigit():
                words.append(token)
    
    return commands, words

def categorize_command(cmd):
    """Enhanced command categorization with more niche commands and languages"""
    cmd_lower = cmd.lower()
    
    # Expanded Navigation commands
    nav_commands = ['cd ', 'ls', 'pwd', 'dir', 'pushd', 'popd', 'll', 'tree', 'exa', 'fd', 'ranger', 'nnn', 'lf']
    if any(x in cmd_lower for x in nav_commands):
        return 'Navigation'
    
    # Expanded File operations
    file_ops = ['cp ', 'mv ', 'rm ', 'mkdir', 'touch', 'chmod', 'chown', 'ln ', 'rsync', 'tar ', 
                'gzip', 'gunzip', 'zip', 'unzip', '7z', 'rename', 'trash', 'shred']
    if any(x in cmd_lower for x in file_ops):
        return 'File Operations'
    
    # Expanded Editors
    editors = ['vim ', 'nano ', 'emacs', 'code ', 'subl ', 'gedit', 'pico', 'vi', 'micro', 'kate', 
               'atom', 'neovim', 'nano', 'ed', 'sed ', 'awk ']
    if any(x in cmd_lower for x in editors):
        return 'Editors'
    
    # Expanded Version control
    vcs = ['git ', 'hg ', 'svn ', 'fossil', 'bzr', 'cvs', 'darcs', 'git-lfs', 'git-flow']
    if any(x in cmd_lower for x in vcs):
        return 'Version Control'
    
    # Expanded Package management
    package_managers = ['apt', 'yum', 'dnf', 'pacman', 'brew', 'pip ', 'npm ', 'snap', 'flatpak', 
                        'zypper', 'port', 'apk', 'dpkg', 'rpm', 'gem', 'cargo', 'go ', 'dotnet']
    if any(x in cmd_lower for x in package_managers):
        return 'Package Management'
    
    # Expanded System monitoring
    system_monitors = ['top', 'htop', 'ps ', 'kill', 'df ', 'du ', 'free', 'btop', 'glances', 'nmon', 
                      'iotop', 'iftop', 'nethogs', 'vmstat', 'iostat', 'dstat', 'sar', 'mpstat', 'pidstat']
    if any(x in cmd_lower for x in system_monitors):
        return 'System Monitoring'
    
    # Expanded Network
    network_commands = ['ssh ', 'scp ', 'ping', 'curl', 'wget', 'ifconfig', 'ip ', 'sftp', 'ftp', 'telnet', 
                        'netstat', 'ss', 'traceroute', 'tracepath', 'mtr', 'dig', 'nslookup', 'nmcli', 'iwconfig']
    if any(x in cmd_lower for x in network_commands):
        return 'Network'
    
    # Programming Languages
    languages = {
        'Python': ['python', 'pip', 'py ', 'python3', 'python2', 'pylint', 'pyflakes', 'mypy', 'black'],
        'Java': ['java ', 'javac', 'mvn ', 'gradle', 'ant ', 'jbang', 'groovy'],
        'Rust': ['rustc', 'cargo', 'rustup', 'rustfmt', 'clippy'],
        'C/C++': ['gcc', 'g++', 'clang', 'make ', 'cmake', 'ninja', 'gdb', 'lldb', 'valgrind', 'cpp'],
        'C#': ['dotnet', 'mono', 'msbuild', 'csc'],
        'JavaScript': ['node ', 'npm ', 'yarn', 'deno', 'tsc', 'bun'],
        'Go': ['go ', 'gofmt', 'golangci-lint'],
        'Ruby': ['ruby ', 'gem ', 'rake', 'bundle'],
        'PHP': ['php ', 'composer', 'phpunit'],
        'Shell': ['bash ', 'sh ', 'zsh ', 'fish ', 'dash', 'ksh'],
        'Assembly': ['as ', 'nasm', 'yasm', 'objdump', 'gdb'],
        'R': ['r ', 'rscript', 'radian'],
        'Perl': ['perl ', 'cpan'],
        'Haskell': ['ghc', 'ghci', 'stack', 'cabal'],
        'Lua': ['lua ', 'luac'],
        'Dart': ['dart ', 'flutter'],
        'Scala': ['scala ', 'scalac'],
        'Kotlin': ['kotlin', 'kotlinc'],
        'Swift': ['swift ', 'swiftc']
    }
    
    for lang, keywords in languages.items():
        if any(x in cmd_lower for x in keywords):
            return f'Language: {lang}'
    
    # Databases
    databases = ['mysql', 'psql', 'sqlite3', 'mongo', 'redis-cli', 'sqlcmd', 'clickhouse-client', 
                 'influx', 'cqlsh', 'neo4j', 'arangosh', 'cockroach sql']
    if any(x in cmd_lower for x in databases):
        return 'Databases'
    
    # Containers/Virtualization
    containers = ['docker ', 'podman', 'kubectl', 'oc ', 'ctr', 'nerdctl', 'lxc', 'lxd', 'vagrant', 
                  'virsh', 'qemu', 'lima', 'colima']
    if any(x in cmd_lower for x in containers):
        return 'Containers/Virtualization'
    
    # Shell builtins
    shell_builtins = ['export', 'source', 'alias', 'echo', 'printf', 'read', 'set', 'unset', 'type', 
                      'hash', 'history', 'fc', 'jobs', 'bg', 'fg', 'wait', 'times', 'trap']
    if any(x in cmd_lower for x in shell_builtins):
        return 'Shell Builtins'
    
    # If none match
    return 'Other'

def print_statistics(commands, words, category_counts):
    """Print comprehensive statistics about the command history"""
    total_commands = len(commands)
    unique_commands = len(set(commands))
    total_words = len(words)
    unique_words = len(set(words))
    
    # Calculate most frequent commands
    command_counts = Counter(commands)
    top_commands = command_counts.most_common(5)
    
    # Calculate most frequent words
    word_counts = Counter(words)
    top_words = word_counts.most_common(5)
    
    # Calculate category distribution
    total_categories = sum(category_counts.values())
    top_categories = category_counts.most_common(5)
    
    # Prepare statistics text
    stats = [
        "=== Command History Statistics ===",
        f"Total commands analyzed: {total_commands}",
        f"Unique commands: {unique_commands} ({unique_commands/total_commands:.1%})",
        f"Total words/tokens: {total_words}",
        f"Unique words/tokens: {unique_words} ({unique_words/total_words:.1%})",
        "",
        "Top 5 Commands:",
        *[f"  {cmd[0]}: {cmd[1]} uses ({cmd[1]/total_commands:.1%})" for cmd in top_commands],
        "",
        "Top 5 Keywords:",
        *[f"  {word[0]}: {word[1]} uses ({word[1]/total_words:.1%})" for word in top_words],
        "",
        "Top 5 Categories:",
        *[f"  {cat[0]}: {cat[1]} commands ({cat[1]/total_categories:.1%})" for cat in top_categories],
        "",
        "=== Detailed Category Breakdown ===",
        *[f"{cat[0]}: {cat[1]} commands ({cat[1]/total_categories:.1%})" for cat in category_counts.most_common()],
        "",
        f"Average commands per category: {total_categories/len(category_counts):.1f}",
        f"Most common category: {category_counts.most_common(1)[0][0]} ({category_counts.most_common(1)[0][1]} commands)",
        f"Least common category: {category_counts.most_common()[-1][0]} ({category_counts.most_common()[-1][1]} commands)"
    ]
    
    # Print with wrapping for better readability
    wrapper = textwrap.TextWrapper(width=80, subsequent_indent='  ')
    for line in stats:
        print(wrapper.fill(line))

def visualize_commands(commands, words, top_n=15):
    """Create visualizations with pie charts and statistics"""
    # Count command categories
    categories = [categorize_command(cmd) for cmd in commands]
    category_counts = Counter(categories)
    
    # Count word usage
    word_counts = Counter(words)
    
    # Set up the figure with 3 subplots (2 pie charts and text area)
    plt.figure(figsize=(18, 10))
    
    # Plot top commands by category (pie chart)
    plt.subplot(1, 3, 1)
    top_categories = category_counts.most_common()
    labels = [cat[0] for cat in top_categories]
    sizes = [cat[1] for cat in top_categories]
    
    # Only show labels for categories with >5% of total
    total = sum(sizes)
    def autopct_format(pct):
        return f'{pct:.1f}%' if pct > 5 else ''
    
    plt.pie(sizes, labels=labels, autopct=autopct_format,
            startangle=90, wedgeprops={'edgecolor': 'white'})
    plt.title('Command Categories Distribution')
    plt.axis('equal')
    
    # Plot top keywords (pie chart)
    plt.subplot(1, 3, 2)
    top_words = word_counts.most_common(top_n)
    word_labels = [word[0] for word in top_words]
    word_sizes = [word[1] for word in top_words]
    
    plt.pie(word_sizes, labels=word_labels, autopct='%1.1f%%',
            startangle=90, wedgeprops={'edgecolor': 'white'})
    plt.title(f'Top {top_n} Command Keywords')
    plt.axis('equal')
    
    # Add statistics text
    plt.subplot(1, 3, 3)
    stats_text = [
        f"Total commands: {len(commands)}",
        f"Unique commands: {len(set(commands))}",
        f"Total keywords: {len(words)}",
        f"Unique keywords: {len(set(words))}",
        f"Categories found: {len(category_counts)}",
        "",
        "Top Categories:",
        *[f"- {cat[0]}: {cat[1]} ({cat[1]/total:.1%})" for cat in category_counts.most_common(5)],
        "",
        "Most Used Commands:",
        *[f"- {cmd[0]}: {cmd[1]}" for cmd in Counter(commands).most_common(5)],
        "",
        "Most Used Keywords:",
        *[f"- {word[0]}: {word[1]}" for word in word_counts.most_common(5)]
    ]
    
    plt.text(0.1, 0.95, "\n".join(stats_text), 
             ha='left', va='top', fontfamily='monospace', fontsize=10)
    plt.axis('off')
    plt.title('Command History Statistics')
    
    plt.tight_layout()
    plt.show()
    
    # Print detailed statistics to console
    print("\nDetailed Statistics:")
    print_statistics(commands, words, category_counts)

def main():
    print("Bash History Analyzer - Loading your command history...")
    
    history_text = get_bash_history()
    
    if not history_text:
        print("Failed to get bash history. Trying fallback method...")
        try:
            with open(os.path.expanduser('~/.bash_history'), 'r', errors='ignore') as f:
                history_text = f.read()
        except Exception as e:
            print(f"Error reading .bash_history file: {e}")
            return
    
    commands, words = process_bash_history(history_text)
    
    if not commands:
        print("No valid commands found in the history.")
        return
    
    print(f"\nAnalyzed {len(commands)} commands with {len(words)} keywords.")
    visualize_commands(commands, words)

if __name__ == "__main__":
    import os
    main()