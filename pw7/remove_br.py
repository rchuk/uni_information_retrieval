import re
import sys

def remove_br_tags(filename):
    with open(filename, 'r', encoding='utf-8') as file:
        content = file.read()

    content = re.sub(r'<br\s*\/?>', ' ', content)

    with open(filename, 'w', encoding='utf-8') as file:
        file.write(content)

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python remove_br.py <filename>")
        sys.exit(1)

    filename = sys.argv[1]
    remove_br_tags(filename)
    print(f"Removed <br/> tags from {filename}")
