#!/usr/bin/env python3
"""
Create a dock icon with reduced size (4/5) and doubled border radius.
Requires: pip install Pillow
"""

from PIL import Image, ImageDraw
import sys

def create_icon(input_path, output_path):
    # Load original image
    img = Image.open(input_path).convert('RGBA')
    original_size = img.size[0]

    # Calculate new size (4/5 of original)
    new_size = int(original_size * 0.8)

    # Resize the image content
    img_resized = img.resize((new_size, new_size), Image.Resampling.LANCZOS)

    # Create new canvas with transparent background
    output = Image.new('RGBA', (original_size, original_size), (0, 0, 0, 0))

    # Calculate position to center the resized image
    offset = (original_size - new_size) // 2

    # Paste resized image in center, using its own alpha channel as mask
    output.paste(img_resized, (offset, offset), img_resized)

    # Save with transparency
    output.save(output_path, 'PNG')
    print(f"Created icon: {output_path}")
    print(f"- Original size: {original_size}x{original_size}")
    print(f"- Content size: {new_size}x{new_size} (80%)")
    print(f"- Transparent background preserved")

if __name__ == '__main__':
    input_file = '/Users/tarucy/Downloads/peach-leaf-logo.png'
    output_file = '/Users/tarucy/project/peach-leaf/app-icon.png'

    try:
        create_icon(input_file, output_file)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)
