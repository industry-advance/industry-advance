from PIL import Image
from typing import List, Tuple

CHUNK_X_SIZE = 256
CHUNK_Y_SIZE = 256


def split_map(in_path: str, out_dir: str) -> List[str]:
    img = Image.open(in_path)
    # If the map is already small enough, don't split
    if img.width <= 512 and img.height <= 512:
        ls = list()
        ls.append(in_path)
        return ls
    # TODO: Implement
    crop_coords: List[Tuple[int, int, int, int]] = list()
    img.crop(box=(0, 0, CHUNK_X_SIZE - 1, CHUNK_Y_SIZE - 1))
