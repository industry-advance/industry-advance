
"""
This module takes the path to a sprite or map that is not in a resolution the GBA can render
and pads it to the nearest GBA-native resolution.
The original sprite/map ends up in the top-left corner (because that's where the GBA sprite coordinates refer to).
"""

from PIL import Image
from typing import Tuple, List
import os

SPRITE_SIZES: List[Tuple[int, int]] = [(8, 8), (8, 16), (16, 8), (16, 16), (32, 8), (8, 32), (16, 32), (32, 16), (32, 32), (64, 32), (32, 64), (64, 64)]
MAP_MULTIPLE_SIZE: int = 256

def halve_resolution(in_path: str, out_path: str):
    img = Image.open(in_path)
    x_size, y_size = img.size
    img.resize((x_size // 2, y_size // 2), Image.NEAREST).save(out_path)

def sprite_is_too_large(image_path: str) -> bool:
    img = Image.open(image_path)
    width, height = img.size
    if (width > 64) or (height > 64):
        return True
    return False


def sprite_needs_padding(image_path: str) -> bool:
    img = Image.open(image_path)
    if img.size in SPRITE_SIZES:
        return False
    return True


def pad_sprite(in_path: str, out_path: str):
    with Image.open(in_path) as img:
        width, height = img.size
        target_width, target_height = _get_nearest_sprite_size(width, height)
        _apply_padding(img, out_path, target_width, target_height)


def _get_nearest_sprite_size(image_width: int, image_height: int) -> Tuple[int, int]:
    # Work our way down the list of sprite sizes, from smallest to largest
    for (width, height) in SPRITE_SIZES:
        if (image_width <= width) and (image_height <= height):
            return (width, height)

    raise Exception("Sprite too large")


def _apply_padding(image: Image.Image, out_path: str, nearest_width: int, nearest_height: int):
    background = Image.new("RGBA", (nearest_width, nearest_height), (0, 0, 0, 0))  # Fully transparent image

    background.paste(image, (0, 0))
    background.save(out_path, "PNG")

def _get_nearest_map_size(image_width: int, image_height: int) -> Tuple[int, int]:
    new_width = (image_width + MAP_MULTIPLE_SIZE) - (image_width % MAP_MULTIPLE_SIZE)
    new_height = (image_height + MAP_MULTIPLE_SIZE) - (image_height % MAP_MULTIPLE_SIZE)
    return (new_width, new_height)
        
def pad_map(in_path: str) -> str:
    """
    Pads the map to the nearest multiple of 256 pixels (32 8x8 tiles).
    """
    with Image.open(in_path) as img:
        width, height = img.size
        if (width % MAP_MULTIPLE_SIZE) == 0 and (width % MAP_MULTIPLE_SIZE) == 0:
            return in_path
        else:
            out_path = "{}-p.png".format(os.path.splitext(in_path)[0])
            target_width, target_height = _get_nearest_map_size(width, height)
            _apply_padding(img, out_path, target_width, target_height)
            return out_path
