#!/usr/bin/env python3


# This script converts graphics to Rust source code.
from c2rust import convert

import os
import subprocess
import shutil
import re
from typing import List

GFX_DIR = "src/assets"
SPRITE_DIR = "{}/sprites".format(GFX_DIR)


def init_gfx_dir():

    # Remove the asset directory, as rebuilding assets may mess stuff up
    if os.path.exists(GFX_DIR):
        shutil.rmtree(GFX_DIR)

    os.mkdir(GFX_DIR)

    # Paste in some Rust code to make it a proper module
    with open("{}/mod.rs".format(GFX_DIR), "w+") as f:
        f.write("pub(crate) mod sprites;")

    os.mkdir(SPRITE_DIR)

    with open("{}/mod.rs".format(SPRITE_DIR), "w+") as f:
        f.write("pub(crate) mod palette;\n")
        f.write("pub(crate) mod sprites;")


def convert_sprites():
    # Paths to sprites
    sprite_paths: List[str] = list()
    # Player ships
    sprite_paths.append("Mindustry/core/assets-raw/sprites/mechs/ships/dart-ship.png")
    sprite_paths.append("Mindustry/core/assets-raw/sprites/mechs/ships/glaive-ship.png")
    sprite_paths.append(
        "Mindustry/core/assets-raw/sprites/mechs/ships/javelin-ship.png"
    )
    sprite_paths.append(
        "Mindustry/core/assets-raw/sprites/mechs/ships/trident-ship.png"
    )

    # For now, all sprites are assumed to be part of the same scene, and therefore share the same palette
    all_sprite_paths: str = ""
    for sprite_path in sprite_paths:
        all_sprite_paths = all_sprite_paths + " " + sprite_path

    shared_data_c_path = "{}/palette.c".format(SPRITE_DIR)
    output_c_path = "{}/sprites.c".format(SPRITE_DIR)
    shared_data_rs_path = "{}/palette.rs".format(SPRITE_DIR)
    output_rs_path = "{}/sprites.rs".format(SPRITE_DIR)

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
    convert_sprites()


if __name__ == "__main__":
    main()
