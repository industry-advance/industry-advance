
"""
This module takes the path to a sprite that is not in a resolution the GBA can render
and pads it to the nearest GBA-native resolution.
The original sprite ends up in the top-left corner (because that's where the GBA sprite coordinates refer to).
"""

from PIL import Image
from typing import Tuple, List

SPRITE_SIZES: List[Tuple[int, int]] = [(8, 8), (8, 16), (16, 8), (16, 16), (32, 8), (8, 32), (16, 32), (32, 16), (32, 32), (64, 32), (32, 64), (64, 64)]
"""
This module calculates the next-largest applicable GBA sprite size for the image, if any.
"""


def image_is_too_large(image_path: str) -> bool:
    img = Image.open(image_path)
    width, height = img.size
    if (width > 64) or (height > 64):
        return True
    return False


def needs_padding(image_path: str) -> bool:
    img = Image.open(image_path)
    if img.size in SPRITE_SIZES:
        return False
    return True


def pad_sprite(in_path: str, out_path: str):
    with Image.open(in_path) as img:
        width, height = img.size
        target_width, target_height = _get_nearest_size(width, height)
        _apply_padding(img, out_path, target_width, target_height)


def _get_nearest_size(image_width: int, image_height: int) -> Tuple[int, int]:
    # Work our way down the list of sprite sizes, from smallest to largest
    for (width, height) in SPRITE_SIZES:
        if (image_width <= width) and (image_height <= height):
            return (width, height)

    raise Exception("Sprite too large")


def _apply_padding(image: Image.Image, out_path: str, nearest_width: int, nearest_height: int):
    background = Image.new("RGBA", (nearest_width, nearest_height), (0, 0, 0, 0))  # Fully transparent image

    background.paste(image, (0, 0))
    background.save(out_path, "PNG")
