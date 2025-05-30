import re
from collections import Counter
import matplotlib.pyplot as plt
from wordcloud import WordCloud

def process_bash_history(history_text):
    """Process bash history and extract commands and keywords"""
    # Remove line numbers if present (from 'history' command output)
    lines = [line.strip() for line in history_text.split('\n') if line.strip()]

    # Extract commands (remove timestamps if present and line numbers)
    commands = []
    for line in lines:
        # Handle lines with format: " 123  command"
        if re.match(r'^\s*\d+\s+', line):
            command = re.sub(r'^\s*\d+\s+', '', line)
        # Handle lines with format: "#1234567890" (timestamp)
        elif re.match(r'^#\d+', line):
            continue  # skip timestamp lines
        else:
            command = line
        commands.append(command)

    # Split commands into words/tokens
    words = []
    for cmd in commands:
        # Split on spaces, but keep quoted strings together
        tokens = re.findall(r'(?:[^\s,"]|"(?:\\.|[^"])*")+', cmd)
        for token in tokens:
            # Remove common flags and options
            if token.startswith('-'):
                continue
            # Remove quotes if present
            token = token.strip('"\'')
            # Skip empty tokens and numbers
            if token and not token.isdigit():
                words.append(token.lower())

    return commands, words

def visualize_commands(commands, words, top_n=20):
    """Create visualizations of command usage"""
    # Count command usage
    command_counts = Counter(commands)

    # Count word usage
    word_counts = Counter(words)

    # Set up the figure
    plt.figure(figsize=(15, 10))

    # Plot top commands
    plt.subplot(2, 2, 1)
    top_commands = command_counts.most_common(top_n)
    plt.barh([cmd[0] for cmd in top_commands], [cmd[1] for cmd in top_commands])
    plt.title(f'Top {top_n} Most Used Commands')
    plt.gca().invert_yaxis()

    # Plot top keywords
    plt.subplot(2, 2, 2)
    top_words = word_counts.most_common(top_n)
    plt.barh([word[0] for word in top_words], [word[1] for word in top_words])
    plt.title(f'Top {top_n} Most Used Keywords')
    plt.gca().invert_yaxis()

    # Create word cloud
    plt.subplot(2, 1, 2)
    wordcloud = WordCloud(width=800, height=400, background_color='white').generate_from_frequencies(word_counts)
    plt.imshow(wordcloud, interpolation='bilinear')
    plt.axis('off')
    plt.title('Command Keywords Word Cloud')

    plt.tight_layout()
    plt.show()

def main():
    print("Bash History Analyzer")
    print("Please paste your bash history (from the 'history' command) below.")
    print("Press Ctrl+D (Unix) or Ctrl+Z then Enter (Windows) when finished.")

    # Read input from stdin
    try:
        history_text = []
        while True:
            try:
                line = input()
                history_text.append(line)
            except EOFError:
                break
        history_text = '\n'.join(history_text)
    except KeyboardInterrupt:
        print("\nOperation cancelled by user.")
        return

    if not history_text.strip():
        print("No input provided. Exiting.")
        return

    commands, words = process_bash_history(history_text)

    if not commands:
        print("No valid commands found in the input.")
        return

    print(f"\nProcessed {len(commands)} commands with {len(words)} keywords.")
    visualize_commands(commands, words)

if __name__ == "__main__":
    main()
