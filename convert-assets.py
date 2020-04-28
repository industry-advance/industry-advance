#!/usr/bin/env python3

# This module provides functionality for modifying (padding, resizing) sprites.
import modifysprite

# This module provides functionality for splitting maps.
import splitmap

import os
import pathlib
import subprocess
import tempfile
from typing import List

# Directory containing sprites to be processed
SPRITES_IN_DIR = "Mindustry/core/assets-raw/sprites/"
# Directories to ignore when converting sprites (for example, because they contain huge zone maps we don't need)
SPRITES_IGNORE_SUBDIRS: List[str] = ["zones", "editor", "ui", "effects"]
# Direcoties containing assets which need to be rescaled (halved in resolution) in order to fit well on a GBA screen
SPRITES_RESIZE_SUBDIRS: List[str] = ["blocks", "mechs", "walls"]

# Same for maps
# Note that this directory currently contains maps as .png files, as the mindustry map parser is WIP.
MAPS_IN_DIR = "testmaps"


def get_sprite_paths() -> List[str]:
    # Paths to sprites
    sprite_paths: List[str] = list()

    for root, dirs, files in os.walk(SPRITES_IN_DIR, topdown=True):
        dirs[:] = [d for d in dirs if d not in SPRITES_IGNORE_SUBDIRS]
        for name in files:
            if name.endswith(".png"):
                sprite_paths.append(os.path.join(root, name))
    return sprite_paths


def rescale_sprites_if_needed(in_paths: List[str]) -> List[str]:
    out_paths: List[str] = list()
    resized_sprite_dir: str = tempfile.TemporaryDirectory().name
    for path in in_paths:
        was_resized = False
        # Check whether path is child path of dir that requires resizing
        for subdir_to_resize in SPRITES_RESIZE_SUBDIRS:
            if os.path.realpath(path).startswith(
                os.path.realpath(os.path.join(SPRITES_IN_DIR, subdir_to_resize))
            ):
                out_path: str = os.path.join(resized_sprite_dir, path)
                pathlib.Path(os.path.dirname(out_path)).mkdir(
                    parents=True, exist_ok=True
                )
                print("Resizing sprite {}".format(path))
                modifysprite.halve_resolution(path, out_path)
                out_paths.append(out_path)
                was_resized = True
                break
        if not was_resized:
            out_paths.append(path)
    return out_paths


def pad_sprites_if_needed(in_paths: List[str]) -> List[str]:
    out_paths: List[str] = list()
    padded_sprite_dir: str = tempfile.TemporaryDirectory().name
    for path in in_paths:
        if modifysprite.image_is_too_large(path):
            print("WARNING: Sprite {} too large, skipping".format(path))
            continue
        if modifysprite.needs_padding(path):
            print("Padding sprite {}".format(path))
            out_path: str = os.path.join(padded_sprite_dir, path)
            pathlib.Path(os.path.dirname(out_path)).mkdir(parents=True, exist_ok=True)
            modifysprite.pad_sprite(path, out_path)
            out_paths.append(out_path)
        else:
            out_paths.append(path)

    return out_paths


def convert_sprites(sprite_paths: List[str]):

    # For now, all sprites are assumed to be part of the same scene, and therefore share the same palette
    all_sprite_paths: str = ""
    for sprite_path in sprite_paths:
        all_sprite_paths = all_sprite_paths + " " + sprite_path

    # Run grit
    subprocess.run(
        "grit {}  -fa -ftg -fh! -gT -pS -S sprite_shared -oassets -Oassets -gB8".format(
            all_sprite_paths
        ),
        shell=True,
        check=True,
    )


def get_map_paths() -> List[str]:
    # Paths to sprites
    map_paths: List[str] = list()

    for root, dirs, files in os.walk(MAPS_IN_DIR, topdown=True):
        for name in files:
            if name.endswith(".png"):
                map_paths.append(os.path.join(root, name))
    return map_paths


"""
As the GBA doesn't support infinite-sized maps, this function
splits them into 32x32 tile (256x256 px) chunks.
"""


def split_maps_into_chunks(in_paths: List[str]) -> List[str]:
    out_paths: List[str] = list()
    for path in in_paths:
        split_map_dir: str = tempfile.TemporaryDirectory().name
        pathlib.Path(split_map_dir).mkdir(parents=True, exist_ok=True)
        split_paths = splitmap.split_map(path, split_map_dir)
        out_paths.extend(split_paths)

    return out_paths


def convert_maps(map_paths: List[str]):
    all_map_paths: str = ""
    for map_path in map_paths:
        all_map_paths = all_map_paths + " " + map_path

    # Run grit
    subprocess.run(
        "grit {} -ftg -fh! -fa -gT -gS -pS -m -oassets -Oassets -S map_shared -gB4".format(
            all_map_paths
        ),
        shell=True,
        check=True,
    )


def main():
    print("----Converting sprites...----")
    sprite_paths = get_sprite_paths()
    rescaled_sprite_paths = rescale_sprites_if_needed(sprite_paths)
    padded_sprite_paths = pad_sprites_if_needed(rescaled_sprite_paths)
    convert_sprites(padded_sprite_paths)

    print("----Converting maps...----")
    map_paths = get_map_paths()
    split_map_paths = split_maps_into_chunks(map_paths)
    convert_maps(split_map_paths)


if __name__ == "__main__":
    main()
