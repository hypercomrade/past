import re
import subprocess
from collections import Counter
import matplotlib.pyplot as plt

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
    """Categorize commands into common types"""
    cmd_lower = cmd.lower()
    
    # Navigation commands
    if any(x in cmd_lower for x in ['cd ', 'ls', 'pwd', 'dir', 'pushd', 'popd', 'll']):
        return 'Navigation'
    
    # File operations
    if any(x in cmd_lower for x in ['cp ', 'mv ', 'rm ', 'mkdir', 'touch', 'chmod', 'chown']):
        return 'File Operations'
    
    # Editors
    if any(x in cmd_lower for x in ['vim ', 'nano ', 'emacs', 'code ', 'subl ', 'gedit', 'pico', 'vi']):
        return 'Editors'
    
    # Version control
    if any(x in cmd_lower for x in ['git ', 'hg ', 'svn ']):
        return 'Version Control'
    
    # Package management
    if any(x in cmd_lower for x in ['apt', 'yum', 'dnf', 'pacman', 'brew', 'pip ', 'npm ']):
        return 'Package Management'
    
    # System monitoring
    if any(x in cmd_lower for x in ['top', 'htop', 'ps ', 'kill', 'df ', 'du ', 'free', 'btop', 'glances']):
        return 'System Monitoring'
    
    # Network
    if any(x in cmd_lower for x in ['ssh ', 'scp ', 'ping', 'curl', 'wget', 'ifconfig', 'ip ', 'sftp']):
        return 'Network'
    
    # Python
    if any(x in cmd_lower for x in ['python', 'pip', 'py ', 'python3', 'python2']):
        return 'Python'
    
    # Shell builtins
    if any(x in cmd_lower for x in ['export', 'source', 'alias', 'echo', 'printf']):
        return 'Shell Builtins'
    
    # If none match
    return 'Other'

def visualize_commands(commands, words, top_n=15):
    """Create visualizations with pie charts"""
    # Count command categories
    categories = [categorize_command(cmd) for cmd in commands]
    category_counts = Counter(categories)
    
    # Count word usage
    word_counts = Counter(words)
    
    # Set up the figure
    plt.figure(figsize=(15, 8))
    
    # Plot top commands by category (pie chart)
    plt.subplot(1, 2, 1)
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
    plt.axis('equal')  # Equal aspect ratio ensures pie is drawn as a circle
    
    # Plot top keywords (pie chart)
    plt.subplot(1, 2, 2)
    top_words = word_counts.most_common(top_n)
    word_labels = [word[0] for word in top_words]
    word_sizes = [word[1] for word in top_words]
    
    plt.pie(word_sizes, labels=word_labels, autopct='%1.1f%%',
            startangle=90, wedgeprops={'edgecolor': 'white'})
    plt.title(f'Top {top_n} Command Keywords')
    plt.axis('equal')
    
    plt.tight_layout()
    plt.show()

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