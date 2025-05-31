#!/usr/bin/env python3
import re
import subprocess
from collections import Counter
import matplotlib.pyplot as plt
import numpy as np
from matplotlib import cm
import argparse
import sys
import os
import json

# Set global style parameters (only used if plotting)
plt.style.use('seaborn-v0_8-pastel')
plt.rcParams['font.family'] = 'DejaVu Sans'  # More modern font
plt.rcParams['axes.titlepad'] = 20
plt.rcParams['axes.labelpad'] = 10

def get_bash_history():
    """Get bash history by running the history command"""
    try:
        result = subprocess.run(['bash', '-i', '-c', 'history'], 
                               capture_output=True, text=True, timeout=10)
        if result.returncode == 0:
            return result.stdout
        else:
            print("Error running history command:", result.stderr, file=sys.stderr)
            return None
    except Exception as e:
        print(f"Error getting bash history: {e}", file=sys.stderr)
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
    """Categorize commands into common types"""
    cmd_lower = cmd.lower()
    
    # Navigation commands
    nav_commands = ['cd ', 'ls', 'pwd', 'dir', 'pushd', 'popd', 'll', 'tree', 'exa', 'fd', 'ranger', 'nnn', 'lf']
    if any(x in cmd_lower for x in nav_commands):
        return 'Navigation'
    
    # File operations
    file_ops = ['cp ', 'mv ', 'rm ', 'mkdir', 'touch', 'chmod', 'chown', 'ln ', 'rsync', 'tar ', 
                'gzip', 'gunzip', 'zip', 'unzip', '7z', 'rename', 'trash', 'shred']
    if any(x in cmd_lower for x in file_ops):
        return 'File Ops'
    
    # Editors
    editors = ['vim ', 'nano ', 'emacs', 'code ', 'subl ', 'gedit', 'pico', 'vi', 'micro', 'kate', 
               'atom', 'neovim', 'nano', 'ed', 'sed ', 'awk ']
    if any(x in cmd_lower for x in editors):
        return 'Editors'
    
    # Version control
    vcs = ['git ', 'hg ', 'svn ', 'fossil', 'bzr', 'cvs', 'darcs', 'git-lfs', 'git-flow']
    if any(x in cmd_lower for x in vcs):
        return 'Version Ctrl'
    
    # Package management
    package_managers = ['apt', 'yum', 'dnf', 'pacman', 'brew', 'pip ', 'npm ', 'snap', 'flatpak', 
                        'zypper', 'port', 'apk', 'dpkg', 'rpm', 'gem', 'cargo', 'go ', 'dotnet']
    if any(x in cmd_lower for x in package_managers):
        return 'Pkg Mgmt'
    
    # System monitoring
    system_monitors = ['top', 'htop', 'ps ', 'kill', 'df ', 'du ', 'free', 'btop', 'glances', 'nmon', 
                      'iotop', 'iftop', 'nethogs', 'vmstat', 'iostat', 'dstat', 'sar', 'mpstat', 'pidstat']
    if any(x in cmd_lower for x in system_monitors):
        return 'Sys Monitor'
    
    # Network
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
            return f'Lang: {lang}'
    
    # Databases
    databases = ['mysql', 'psql', 'sqlite3', 'mongo', 'redis-cli', 'sqlcmd', 'clickhouse-client', 
                 'influx', 'cqlsh', 'neo4j', 'arangosh', 'cockroach sql']
    if any(x in cmd_lower for x in databases):
        return 'Databases'
    
    # Containers/Virtualization
    containers = ['docker ', 'podman', 'kubectl', 'oc ', 'ctr', 'nerdctl', 'lxc', 'lxd', 'vagrant', 
                  'virsh', 'qemu', 'lima', 'colima']
    if any(x in cmd_lower for x in containers):
        return 'Containers'
    
    # Shell builtins
    shell_builtins = ['export', 'source', 'alias', 'echo', 'printf', 'read', 'set', 'unset', 'type', 
                      'hash', 'history', 'fc', 'jobs', 'bg', 'fg', 'wait', 'times', 'trap']
    if any(x in cmd_lower for x in shell_builtins):
        return 'Shell Builtins'
    
    # If none match
    return 'Other'

def print_statistics(commands, words, category_counts, output_format='text'):
    """Print comprehensive statistics about the command history"""
    total_commands = len(commands)
    unique_commands = len(set(commands))
    total_words = len(words)
    unique_words = len(set(words))
    
    # Calculate most frequent commands
    command_counts = Counter(commands)
    top_commands = command_counts.most_common(10)
    
    # Calculate most frequent words
    word_counts = Counter(words)
    top_words = word_counts.most_common(10)
    
    # Calculate category distribution
    total_categories = sum(category_counts.values())
    top_categories = category_counts.most_common(10)
    
    if output_format == 'json':
        result = {
            'summary': {
                'total_commands': total_commands,
                'unique_commands': unique_commands,
                'command_variety': unique_commands/total_commands,
                'total_keywords': total_words,
                'unique_keywords': unique_words,
                'keyword_variety': unique_words/total_words
            },
            'top_commands': [{'command': cmd[0], 'count': cmd[1]} for cmd in top_commands],
            'top_words': [{'word': word[0], 'count': word[1]} for word in top_words],
            'top_categories': [{'category': cat[0], 'count': cat[1]} for cat in top_categories],
            'all_categories': {cat[0]: cat[1] for cat in category_counts.most_common()}
        }
        print(json.dumps(result, indent=2))
        return
    
    # Prepare statistics text with box
    stats = [
        "╔════════════════════════════════════════════╗",
        "║          COMMAND HISTORY ANALYSIS          ║",
        "╟────────────────────────────────────────────╢",
        f"║ {'Total commands:':<20} {total_commands:>12,} ║",
        f"║ {'Unique commands:':<20} {unique_commands:>12,} ║",
        f"║ {'Command variety:':<20} {unique_commands/total_commands:>12.1%} ║",
        "╟────────────────────────────────────────────╢",
        f"║ {'Total keywords:':<20} {total_words:>12,} ║",
        f"║ {'Unique keywords:':<20} {unique_words:>12,} ║",
        f"║ {'Keyword variety:':<20} {unique_words/total_words:>12.1%} ║",
        "╟────────────────────────────────────────────╢",
        "║           MOST FREQUENT COMMANDS           ║",
    ]
    
    # Add top commands
    for i, cmd in enumerate(top_commands):
        cmd_text = f"{i+1}. {cmd[0][:30]:<33}{cmd[1]:>5,}"
        stats.append(f"║ {cmd_text} ║")
    
    stats.append("╟────────────────────────────────────────────╢")
    stats.append("║            MOST FREQUENT WORDS             ║")
    
    # Add top words
    for i, word in enumerate(top_words):
        word_text = f"{i+1}. {word[0][:30]:<33}{word[1]:>5,}"
        stats.append(f"║ {word_text} ║")
    
    stats.append("╟────────────────────────────────────────────╢")
    stats.append("║             TOP CATEGORIES                 ║")
    
    # Add top categories
    for i, cat in enumerate(top_categories):
        cat_text = f"{i+1}. {cat[0][:30]:<33}{cat[1]:>5,}"
        stats.append(f"║ {cat_text} ║")
    
    stats.append("╚════════════════════════════════════════════╝")
    
    # Print with colored output if available and not json
    try:
        from termcolor import colored
        print(colored('\n'.join(stats), 'cyan', attrs=['bold']))
    except ImportError:
        print('\n'.join(stats))

def create_donut_chart(ax, data, title, colors=None):
    """Create a donut chart with better label placement"""
    labels = [x[0] for x in data]
    sizes = [x[1] for x in data]
    total = sum(sizes)
    
    # Generate colors if not provided
    if not colors:
        colors = cm.Pastel1(np.linspace(0, 1, len(labels)))
    
    # Create pie chart (donut)
    wedges, texts, autotexts = ax.pie(
        sizes, 
        labels=labels,
        autopct=lambda p: f'{p:.1f}%' if p >= 5 else '',
        startangle=90,
        wedgeprops=dict(width=0.5, edgecolor='white'),
        colors=colors,
        textprops={'fontsize': 9, 'fontweight': 'bold'},
        pctdistance=0.85
    )
    
    # Improve label placement
    for text in texts:
        text.set_fontsize(8)
        text.set_fontweight('normal')
    
    # Add title
    ax.set_title(title, fontsize=12, fontweight='bold', pad=20)
    
    # Equal aspect ratio ensures the pie is drawn as a circle
    ax.axis('equal')
    
    # Add center text with total count
    centre_circle = plt.Circle((0,0), 0.3, fc='white')
    ax.add_artist(centre_circle)
    ax.text(0, 0, f"Total:\n{total}", ha='center', va='center', fontsize=10, fontweight='bold')

def create_bar_chart(ax, data, title, color):
    """Create a horizontal bar chart"""
    labels = [x[0] for x in data]
    values = [x[1] for x in data]
    y_pos = np.arange(len(labels))
    
    bars = ax.barh(y_pos, values, color=color, edgecolor='white')
    ax.set_yticks(y_pos)
    ax.set_yticklabels(labels, fontsize=9)
    ax.invert_yaxis()
    ax.set_title(title, fontsize=12, fontweight='bold', pad=15)
    
    # Add value labels
    for bar in bars:
        width = bar.get_width()
        ax.text(width + max(values)*0.01, bar.get_y() + bar.get_height()/2,
                f'{width:,}', va='center', fontsize=8)
    
    # Remove spines and ticks
    ax.spines['top'].set_visible(False)
    ax.spines['right'].set_visible(False)
    ax.spines['bottom'].set_visible(False)
    ax.xaxis.set_ticks_position('none')

def generate_visualizations(commands, words, output_file=None, top_n=15):
    """Create professional visualizations with multiple chart types"""
    # Count command categories
    categories = [categorize_command(cmd) for cmd in commands]
    category_counts = Counter(categories).most_common()
    
    # Count word usage
    word_counts = Counter(words).most_common(top_n)
    
    # Count command usage
    command_counts = Counter(commands).most_common(top_n)
    
    # Set up the figure with a grid layout
    fig = plt.figure(figsize=(18, 12), facecolor='#f5f5f5')
    fig.suptitle('Bash Command History Analysis', fontsize=16, fontweight='bold', y=0.98)
    
    # Create grid layout
    gs = fig.add_gridspec(2, 3, height_ratios=[3, 1])
    
    # Donut chart for categories
    ax1 = fig.add_subplot(gs[0, 0])
    create_donut_chart(ax1, category_counts, 'Command Categories')
    
    # Donut chart for keywords
    ax2 = fig.add_subplot(gs[0, 1])
    create_donut_chart(ax2, word_counts, f'Top {top_n} Keywords')
    
    # Bar chart for top commands
    ax3 = fig.add_subplot(gs[0, 2])
    create_bar_chart(ax3, command_counts, f'Top {top_n} Commands', '#8da0cb')
    
    # Statistics text
    ax4 = fig.add_subplot(gs[1, :])
    stats_text = [
        f"Total Commands: {len(commands):,} | Unique: {len(set(commands)):,}",
        f"Total Keywords: {len(words):,} | Unique: {len(set(words)):,}",
        f"Categories Found: {len(category_counts)} | Most Used: {category_counts[0][0]} ({category_counts[0][1]:,})",
        f"Analysis Date: {subprocess.getoutput('date')}"
    ]
    ax4.text(0.5, 0.5, '\n'.join(stats_text), 
             ha='center', va='center', fontsize=11, 
             bbox=dict(facecolor='white', alpha=0.8, edgecolor='lightgray', boxstyle='round'))
    ax4.axis('off')
    
    plt.tight_layout()
    plt.subplots_adjust(top=0.92, hspace=0.3)
    
    # Add watermark
    fig.text(0.95, 0.05, 'Bash History Analyzer', 
             fontsize=12, color='gray', ha='right', va='bottom', alpha=0.5)
    
    if output_file:
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        print(f"Visualization saved to {output_file}", file=sys.stderr)
    else:
        plt.show()

def main():
    parser = argparse.ArgumentParser(
        description='Analyze bash command history with various output options',
        formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )
    parser.add_argument('-f', '--file', help='Use a specific history file instead of live bash history')
    parser.add_argument('-o', '--output', help='Output file for visualization (PNG, JPG, SVG, PDF)')
    parser.add_argument('-n', '--top-n', type=int, default=15, 
                       help='Number of top commands/words to display')
    parser.add_argument('-j', '--json', action='store_true', 
                       help='Output results in JSON format')
    parser.add_argument('-v', '--visualize', action='store_true', 
                       help='Generate visualizations (interactive or to file if --output specified)')
    parser.add_argument('-q', '--quiet', action='store_true', 
                       help='Suppress all non-essential output (except JSON if requested)')
    
    args = parser.parse_args()
    
    if not args.quiet:
        print("Bash History Analyzer - Loading your command history...", file=sys.stderr)
    
    if args.file:
        try:
            with open(os.path.expanduser(args.file), 'r', errors='ignore') as f:
                history_text = f.read()
        except Exception as e:
            print(f"Error reading history file: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        history_text = get_bash_history()
        if not history_text:
            if not args.quiet:
                print("Failed to get live bash history. Trying fallback method...", file=sys.stderr)
            try:
                with open(os.path.expanduser('~/.bash_history'), 'r', errors='ignore') as f:
                    history_text = f.read()
            except Exception as e:
                print(f"Error reading .bash_history file: {e}", file=sys.stderr)
                sys.exit(1)
    
    commands, words = process_bash_history(history_text)
    
    if not commands:
        print("No valid commands found in the history.", file=sys.stderr)
        sys.exit(1)
    
    if not args.quiet:
        print(f"\nAnalyzed {len(commands)} commands with {len(words)} keywords.", file=sys.stderr)
    
    # Get categories for statistics
    categories = [categorize_command(cmd) for cmd in commands]
    category_counts = Counter(categories)
    
    # Handle output options
    if args.json:
        print_statistics(commands, words, category_counts, output_format='json')
    elif not args.quiet:
        print_statistics(commands, words, category_counts)
    
    # Handle visualization
    if args.visualize or args.output:
        if args.output and not args.output.lower().endswith(('.png', '.jpg', '.jpeg', '.svg', '.pdf')):
            print("Error: Output file must have .png, .jpg, .svg, or .pdf extension", file=sys.stderr)
            sys.exit(1)
        
        try:
            generate_visualizations(commands, words, output_file=args.output, top_n=args.top_n)
        except Exception as e:
            print(f"Error generating visualization: {e}", file=sys.stderr)
            sys.exit(1)

if __name__ == "__main__":
    main()