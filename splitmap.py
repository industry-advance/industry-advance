from PIL import Image
from typing import List, Tuple
import os

CHUNK_X_SIZE = 256
CHUNK_Y_SIZE = 256


def split_map(in_path: str, out_dir: str) -> List[str]:
    img = Image.open(in_path)
    # Map must be multiple of 256x256 pixels large
    if img.width % 256 != 0 or img.height % 256 != 0:
        raise Exception("Map not multiple of 256x256 pixels")

    # If the map is already small enough, don't split
    if img.width == 256 and img.height == 256:
        ls = list()
        ls.append(in_path)
        return ls

    # Jump forward 512 pixels in the image size before each cut (until rest < 512)
    out_paths = list()
    curr_x = 0
    curr_y = 0
    chunk_num = 0
    while curr_x <= img.width:
        while curr_y <= img.height:
            top_left_x = curr_x
            top_left_y = curr_y
            bottom_right_x = curr_x + CHUNK_X_SIZE
            bottom_right_y = curr_y + CHUNK_Y_SIZE
            chunk = img.crop(
                box=(top_left_x, top_left_y, bottom_right_x, bottom_right_y)
            )
            out_path = os.path.join(
                out_dir,
                "{}_{}.png".format(
                    os.path.basename(os.path.splitext(in_path)[0]), chunk_num
                ),
            )
            chunk.save(out_path)
            out_paths.append(out_path)
            chunk_num = chunk_num + 1
            curr_y = curr_y + CHUNK_Y_SIZE
        curr_x = curr_x + CHUNK_X_SIZE
    return out_paths
