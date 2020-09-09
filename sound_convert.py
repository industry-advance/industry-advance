from typing import List
from pathlib import Path
import tempfile
import os

import gbfs_utils

import ffmpeg

SAMPLE_RATE = 18157

def ogg_to_gba_wav(path: Path, out_path: Path):
    audio = ffmpeg.input(path).audio
    out = ffmpeg.output(audio, filename=out_path, ar="{}".format(SAMPLE_RATE), acodec="pcm_u8", ac="1")
    ffmpeg.run(out)
    # Because samples are read one word at a time, the file size has to be
    # a multiple of 4

    if out_path.stat().st_size % 4 != 0:
        # If it isn't pad it (probably not the best way to avoid artifacts...)
        print("Padding file {} with samples".format(out_path.name))
        pad_len = out_path.stat().st_size % 4
        contents = bytearray(out_path.read_bytes())
        for _ in range(0, pad_len):
            contents.extend([0x0])
        contents_bytes = bytes(contents)
        out_path.write_bytes(contents_bytes)


# Convert all the audio files in the given directories to a GBFS archive
# containing GBA direct sound data.
def sound_dirs_to_gbfs(dirs: List[Path], dest_archive: Path):
    # Get a list of all audio files
    in_files: List[Path] = []
    for d in dirs:
        in_files.extend(
            [
                entry
                for entry in d.iterdir()
                if entry.is_file() and entry.suffix == ".ogg"
            ]
        )

    # Convert each one in a temporary directory
    temp_dir: Path = Path(tempfile.TemporaryDirectory().name).resolve()
    Path(temp_dir).mkdir(parents=True, exist_ok=True)
    old_workdir = os.getcwd()  # Want to return here later
    os.chdir(temp_dir)
    for file in in_files:
        out_file: Path = (temp_dir / file.name).with_suffix(".wav").resolve()
        print("OUT: {}".format(out_file))
        ogg_to_gba_wav(file, out_file)
        gbfs_utils.insert(dest_archive, out_file)
        out_file.unlink()

    # Clean up
    os.chdir(old_workdir)
    os.rmdir(temp_dir)

