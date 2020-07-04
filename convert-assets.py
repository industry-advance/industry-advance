#!/usr/bin/env python3

"""
Script for converting assets to GBA-friendly formats
"""

# This module provides functionality for padding the size of graphical assets.
import pad

# This module provides functionality for splitting maps.
import splitmap

# This module provides conversion of TTF fonts to PNG
import preparefont

# This module converts Mindustry's maps to a more suitable format
import parse_save

# This module makes it easier to deal with the GBFS format, with tooling that is
# surprisingly crippled given the format's simplicity.
import gbfs_utils

import os
import pathlib
import subprocess
import tempfile
from time import sleep

import hashlib

from dataclasses import dataclass
from dataclasses_json import dataclass_json
from typing import List, Tuple

from PIL import Image

# Directory containing sprites to be processed
SPRITES_IN_DIRS = ["Mindustry/core/assets-raw/sprites/", "assets"]
# Directories to ignore when converting sprites (for example, because they contain huge zone maps we don't need
SPRITES_IGNORE_SUBDIRS: List[str] = ["zones", "editor", "ui", "effects"]
# Direcoties containing assets which need to be rescaled (halved in resolution) in order to fit well on a GBA screen
SPRITES_RESIZE_SUBDIRS: List[str] = ["blocks", "mechs", "walls"]

CURRENTLY_USED_SPRITES: List[str] = ["containerTiles.png", "copper-wall.png", "cursor.png", "dart-ship.png", "mechanical-drill.png"]

# Same for maps
# Note that this directory currently contains maps as .png files, as the mindustry map parser is WIP.
MAPS_IN_DIR = "Mindustry/core/assets/maps/"

last_identical_checksum="dc09b91091881808048dd306cae18f7b88c7ed2824d2d3bf08e3339ab1e36acb2905c8ba843513f4fa7b2aabb04cde5d9d7475f8b77bdfe69c2866c8f606b28a"
now_differs=False
checksum_counter = 0

# Path to font which should be included
TTF_FONT_PATH = "Px437_IBM_BIOS.ttf"
# Path to final archive
OUT_PATH = "assets.gbfs"
# Path to intermediate archives, needed because grit (and the GBFS CLI tools) are too stupid to
# append stuff.
OUT_PATH_TMP = "tmp-assets.gbfs"


def get_sprite_paths() -> List[str]:
    # Paths to sprites
    sprite_paths: List[str] = list()
    for top_root in SPRITES_IN_DIRS:
        for root, dirs, files in os.walk(top_root, topdown=True):
            dirs[:] = [d for d in dirs if d not in SPRITES_IGNORE_SUBDIRS]
            for name in files:
                if name.endswith(".png") and name in CURRENTLY_USED_SPRITES:
                    sprite_paths.append(os.path.join(root, name))
                else:
                    print("Not used: " + name)
    sprite_paths.sort()
    print(sprite_paths)
    return sprite_paths


def rescale_sprites_if_needed(in_paths: List[str]) -> List[str]:
    out_paths: List[str] = list()
    resized_sprite_dir: str = tempfile.TemporaryDirectory().name
    for path in in_paths:
        was_resized = False
        # Check whether path is child path of dir that requires resizing
        for subdir_to_resize in SPRITES_RESIZE_SUBDIRS:
            if os.path.realpath(path).startswith(
                os.path.realpath(os.path.join(SPRITES_IN_DIRS[0], subdir_to_resize))
            ):
                out_path: str = os.path.join(resized_sprite_dir, path)
                pathlib.Path(os.path.dirname(out_path)).mkdir(
                    parents=True, exist_ok=True
                )
                print("Resizing sprite {}".format(path))
                pad.halve_resolution(path, out_path)
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
        if pad.sprite_is_too_large(path):
            print("WARNING: Sprite {} too large, skipping".format(path))
            continue
        if pad.sprite_needs_padding(path):
            print("Padding sprite {}".format(path))
            out_path: str = os.path.join(padded_sprite_dir, path)
            pathlib.Path(os.path.dirname(out_path)).mkdir(parents=True, exist_ok=True)
            pad.pad_sprite(path, out_path)
            out_paths.append(out_path)
        else:
            out_paths.append(path)

    return out_paths


def convert_sprites(sprite_paths: List[str]):
    global checksum_counter
    sprite_paths.sort()
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
    os.system("sha512sum "+OUT_PATH+" | tee -a checksums.txt")
    checksum_counter += 1

def convert_fonts():
    global checksum_counter
    temp_dir: str = tempfile.TemporaryDirectory().name
    pathlib.Path(temp_dir).mkdir(parents=True, exist_ok=True)
    char_file = pathlib.Path(temp_dir).joinpath("font_chars.txt")
    img_file = pathlib.Path(temp_dir).joinpath("font.png")
    preparefont.convert_ttf_font(TTF_FONT_PATH, char_file, img_file)
    # Insert character list into FS (they're ordered in the same order as in the img)
    # NOTE: It's important that the insertion happens here, because the gbfs tool does not support appending
    subprocess.run(check=True, args=["gbfs", OUT_PATH, char_file])
    os.system("sha512sum "+OUT_PATH+" | tee -a checksums.txt")
    checksum_counter += 1
    # Run grit to actually convert glyphs
    subprocess.run(
        "grit {} -ftg -fh! -fa -tc -gT -pS -m! -mR! -oassets -Oassets -S font_shared -gB4".format(
            img_file
        ),
        shell=True,
        check=True,
    )
    os.system("sha512sum "+OUT_PATH+" | tee -a checksums.txt")
    checksum_counter += 1
    


def get_map_paths() -> List[str]:
    map_paths: List[str] = list()

    for root, dirs, files in os.walk(MAPS_IN_DIR, topdown=True):
        for name in files:
            if name.endswith(".msav"):
                map_paths.append(os.path.join(root, name))
    return map_paths


def split_maps_into_chunks(in_paths: List[str]) -> List[str]:
    """
    As the GBA doesn't support infinite-sized maps, this function
    splits them into 32x32 tile (256x256 px) chunks.
    """
    out_paths: List[str] = list()
    for path in in_paths:
        split_map_dir: str = tempfile.TemporaryDirectory().name
        pathlib.Path(split_map_dir).mkdir(parents=True, exist_ok=True)
        split_paths = splitmap.split_map(path, split_map_dir)
        out_paths.extend(split_paths)

    return out_paths


def pad_maps(map_paths: List[str]) -> List[str]:
    """
    Pads the map to multiple of 256x256 pixels and returns paths of padded images.
    """
    new_paths = list()
    for m in map_paths:
        new_paths.append(pad.pad_map(m))
    return new_paths


def convert_maps_via_grit(map_paths: List[Tuple[str, List[str]]]):
    """
    Takes a whole bunch of maps as lists of their 32x32 fragments and converts to GBA format.
    """

    def convert_single_map_via_grit(map_name: str, map_fragment_paths: List[str]):
        global checksum_counter
        """
        Takes an array of 32x32 tile slices of a map, and converts them to GBA format.
        WARNING: Do not use on more than 1 map at once, or your colors will be messed up
        due to limited palette size.
        """

        # Grit expects paths as a long list of strings
        all_fragment_paths: str = ""
        for fragment_path in map_fragment_paths:
            all_fragment_paths = all_fragment_paths + " '" + fragment_path + "'"

        print(
            "Packing map named {} with fragments {}".format(
                map_name, all_fragment_paths
            )
        )
        # Because grit is too stupid to append, we need to let it generate a new assets archive
        # then unpack ours and add stuff to it by copying from grit's archive.
        # Run grit
        subprocess.run(
            "grit {} -ftg -fh! -fa -gT -gS -pS -m -o{} -O{} -S map_{}_shared -gB4".format(
                all_fragment_paths, OUT_PATH, OUT_PATH, map_name
            ),
            shell=True,
            check=True,
        )
        print(OUT_PATH)
        os.system("sha512sum "+OUT_PATH+" | tee -a checksums.txt")
        checksum_counter += 1
    map_paths.sort()
    print(map_paths)
    for (map_name, fragmented_map) in map_paths:
        convert_single_map_via_grit(map_name, fragmented_map)


def convert_mindustry_maps_to_png(
    map_paths: List[str],
) -> Tuple[List[str], List[Tuple[int, int, str]]]:
    """
    Converts .msav maps to PNGs.
    Returns tuple containing list of PNG filenames, as well as another list of tuples
    containing width, height and name of each map.
    """
    # Maps that we can't parse (yet)
    # Usually because the map format version is unsupported
    map_blacklist = ["Mindustry/core/assets/maps/shoreline.msav", "Mindustry/core/assets/maps/frozenForest.msav"]
    metadata: List[Tuple[int, int, str]] = list()
    png_paths: List[str] = list()
    for m in map_paths:
        if m in map_blacklist:
            print("Blacklisted map, returning nothing for map")
            continue
        print("Converting map: {}".format(m))
        (width, height, name, png_path) = parse_save.map_file_to_map(m)
        png_paths.append(png_path)
        print(png_path)
        metadata.append((width, height, name))
    return (png_paths, metadata)


# These are used for JSON serialization
# Structure mirrors the one seen in map.rs
@dataclass_json
@dataclass
class MapChunk:
    filename: str


@dataclass_json
@dataclass
class MapEntry:
    name: str
    height: int
    width: int
    chunks: List[MapChunk]


@dataclass_json
@dataclass
class Maps:
    maps: List[MapEntry]


def convert_maps():
    global checksum_counter
    """
    Converts maps to industry-advance format.
    """
    maps: Maps = Maps(maps=list())
    map_paths = get_map_paths()
    (map_png_paths, metadata) = convert_mindustry_maps_to_png(map_paths)
    print(map_png_paths)
    padded_map_png_paths = pad_maps(map_png_paths)
    print(padded_map_png_paths)
    split_map_png_paths: List[Tuple[str, List[str]]] = list()
    for i, map_png in enumerate(padded_map_png_paths):
        split_map_paths = split_maps_into_chunks([map_png])
        map_name = metadata[i][2]
        split_map_png_paths.append((map_name, split_map_paths))

        map_chunks: List[MapChunk] = list()
        # We assume a particular grit naming scheme here
        for split in split_map_paths:
            # For whatever reason, grit replaces "-" with "_"
            grit_filename: str = "{}Map".format(
                os.path.splitext(os.path.basename(split))[0].replace("-", "_")
            )
            map_chunks.append(MapChunk(filename=grit_filename))

        # Get size of map (in tiles)
        img = Image.open(map_png)
        width, height = img.size
        width = width // 32
        height = height // 32
        map_entry: MapEntry = MapEntry(
            width=width, height=height, name=metadata[i][2], chunks=map_chunks,
        )
        maps.maps.append(map_entry)

    convert_maps_via_grit(split_map_png_paths)
    maps.maps.sort(key=lambda obj: obj.name)
    # JSON file containing map metadata
    with open("maps.json", "w") as f:
        print(maps.to_json())
        f.write(maps.to_json())
    gbfs_utils.insert(OUT_PATH, "maps.json")
    os.system("sha512sum "+OUT_PATH+" | tee -a checksums.txt")
    checksum_counter += 1

def main():
    print("----Converting font...----")
    convert_fonts()

    print("----Converting sprites...----")
    sprite_paths = get_sprite_paths()
    rescaled_sprite_paths = rescale_sprites_if_needed(sprite_paths)
    padded_sprite_paths = pad_sprites_if_needed(rescaled_sprite_paths)
    convert_sprites(padded_sprite_paths)

    print("----Converting maps...----")
    convert_maps()


if __name__ == "__main__":
    main()
