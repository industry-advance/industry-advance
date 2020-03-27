#!/usr/bin/env python3


# This script converts graphics to Rust source code.
from c2rust import convert

# This module provides functionality for padding sprites.
import padsprite

import os
import pathlib
import subprocess
import shutil
import tempfile
import re
from typing import List

# Directory containing sprites to be processed
SPRITES_IN_DIR = "Mindustry/core/assets-raw/sprites/"
# Directories to ignore when converting sprites (for example, because they contain huge zone maps we don't need)
SPRITES_IGNORE_SUBDIRS: List[str] = ["zones", "editor", "ui", "effects"]

# Directories to emit Rust source code to
ASSETS_OUT_DIR = "src/assets"
SPRITES_OUT_DIR = "{}/sprites".format(ASSETS_OUT_DIR)


def init_gfx_dir():

    # Remove the asset directory, as rebuilding assets may mess stuff up
    if os.path.exists(ASSETS_OUT_DIR):
        shutil.rmtree(ASSETS_OUT_DIR)

    os.mkdir(ASSETS_OUT_DIR)

    # Paste in some Rust code to make it a proper module
    with open("{}/mod.rs".format(ASSETS_OUT_DIR), "w+") as f:
        f.write("#[allow(dead_code)]")  # So rustc doesn't nag about unused assets
        f.write("pub(crate) mod sprites;")

    os.mkdir(SPRITES_OUT_DIR)

    with open("{}/mod.rs".format(SPRITES_OUT_DIR), "w+") as f:
        f.write("pub(crate) mod palette;\n")
        f.write("pub(crate) mod sprites;")


def get_sprite_paths() -> List[str]:
    # Paths to sprites
    sprite_paths: List[str] = list()

    for root, dirs, files in os.walk(SPRITES_IN_DIR, topdown=True):
        dirs[:] = [d for d in dirs if d not in SPRITES_IGNORE_SUBDIRS]
        for name in files:
            if name.endswith(".png"):
                sprite_paths.append(os.path.join(root, name))
    print(sprite_paths)
    return sprite_paths


def pad_sprites_if_needed(in_paths: List[str]) -> List[str]:
    out_paths: List[str] = list()
    padded_sprite_dir: str = tempfile.TemporaryDirectory().name
    for path in in_paths:
        if padsprite.image_is_too_large(path):
            print("WARNING: Sprite {} too large, skipping".format(path))
            continue
        if padsprite.needs_padding(path):
            print("Padding sprite {}".format(path))
            out_path: str = os.path.join(padded_sprite_dir, path)
            pathlib.Path(os.path.dirname(out_path)).mkdir(parents=True, exist_ok=True)
            padsprite.pad_sprite(path, out_path)
            out_paths.append(out_path)
        else:
            out_paths.append(path)

    return out_paths


def convert_sprites(sprite_paths: List[str]):

    # For now, all sprites are assumed to be part of the same scene, and therefore share the same palette
    all_sprite_paths: str = ""
    for sprite_path in sprite_paths:
        all_sprite_paths = all_sprite_paths + " " + sprite_path

    shared_data_c_path = "{}/palette.c".format(SPRITES_OUT_DIR)
    output_c_path = "{}/sprites.c".format(SPRITES_OUT_DIR)
    shared_data_rs_path = "{}/palette.rs".format(SPRITES_OUT_DIR)
    output_rs_path = "{}/sprites.rs".format(SPRITES_OUT_DIR)

    # Run grit
    subprocess.run(
        "grit {}  -fa -ftc -fh! -gT -pS -o{} -O{} -gB8".format(
            all_sprite_paths, output_c_path, shared_data_c_path
        ),
        shell=True,
        check=True,
    )

    # Convert to Rust code
    convert(shared_data_c_path, shared_data_rs_path)
    convert(output_c_path, output_rs_path)

    # Grit is misleading in that it names the palette array after the first file passed to it,
    # despite it containing colors for all files.
    # Therefore, we fix that here with some regex magic.
    exp = re.compile(r"(const.*PAL)", flags=re.MULTILINE)

    lines: List[str] = list()
    with open(shared_data_rs_path, "r") as f:
        # Read in entire file, modify, then overwrite
        for line in f:
            lines.append(line)

    modified_lines: List[str] = list()
    for line in lines:
        if exp.search(line):
            modified_lines.append(exp.sub("const PAL", line))
        else:
            modified_lines.append(line)

    with open(shared_data_rs_path, "w+") as f:
        f.writelines(modified_lines)


def main():
    init_gfx_dir()
    sprite_paths = get_sprite_paths()
    padded_sprite_paths = pad_sprites_if_needed(sprite_paths)
    convert_sprites(padded_sprite_paths)


if __name__ == "__main__":
    main()
