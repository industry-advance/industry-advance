
"""
This module converts a TTF bitmap font consisting of 8x8 glyphs and converts them to individual PNGs. 
"""
from PIL import Image, ImageFont, ImageDraw
from typing import Dict
from fontTools.ttLib import TTFont

# Width and height of each glyph
GLYPH_SIZE = 8 

# TODO: Remove

def _get_glyphs(path: str) -> str:
    ttf = TTFont(path)
    chars: str = ""
    for table in ttf["cmap"].tables:
        for table_unicode_items in table.cmap.items():
            chars = chars + chr(table_unicode_items[0])
    # Deduplicate string
    chars = "".join(dict.fromkeys(chars))
    ttf.close()
    return chars


def convert_ttf_font(ttf_path: str, char_file_path: str, font_img_path: str):
    """
    Converts an 8x8 bitmap monochrome monospace font into a png.
    """

    font = ImageFont.truetype(ttf_path, size=GLYPH_SIZE, encoding="unic")

    chars = _get_glyphs(ttf_path)
    im = Image.new(size=(GLYPH_SIZE * len(chars), GLYPH_SIZE), mode="RGBA")
    draw = ImageDraw.Draw(im)
    for i, char in enumerate(chars):
        draw.text((i * GLYPH_SIZE, 0), char, (255, 255, 255), font=font)
    with open(char_file_path, "w") as f:
        f.write(chars)
    im.save(font_img_path)
