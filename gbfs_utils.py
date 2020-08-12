"""
This module contains some convenience functions for working with GBFS archives.
"""

import tempfile
import pathlib
from pathlib import Path
import os
import shutil
import subprocess


def insert(gbfs_path: Path, file_path: Path):
    """
    Insert a file into the given, preexisting GBFS archive.
    Given paths must be absolute.
    """
    # The GBFS distribution provides no tooling for appending files to an archive,
    # so we have to extract and repack instead.
    temp_dir: str = tempfile.TemporaryDirectory().name
    pathlib.Path(temp_dir).mkdir(parents=True, exist_ok=True)
    old_workdir = os.getcwd()
    absolute_gbfs_path = os.path.abspath(gbfs_path)
    absolute_file_path = os.path.abspath(file_path)
    os.chdir(temp_dir)
    subprocess.run(check=True, args=["ungbfs", absolute_gbfs_path])
    shutil.copy2(absolute_file_path, temp_dir)
    # Pack back up
    all_files = os.listdir(temp_dir)
    # Have to do this in 2 steps, or we get "Invalid cross-device link"
    subprocess.run(check=True, args=["gbfs", "temp.gbfs"] + all_files)
    shutil.copy2("temp.gbfs", absolute_gbfs_path)

    os.chdir(old_workdir)


def merge(archive_1_path: str, archive_2_path: str, archive_out_path: str):
    """
    Merge the given archives together into the archive_out archive.
    """

    # Create a tempdir for this mess
    temp_dir: str = tempfile.TemporaryDirectory().name
    pathlib.Path(temp_dir).mkdir(parents=True, exist_ok=True)
    old_workdir = os.getcwd()

    absolute_archive_1_path = os.path.abspath(archive_1_path)
    absolute_archive_2_path = os.path.abspath(archive_2_path)
    # Extract archive 1 into temporary dir
    os.chdir(temp_dir)
    subprocess.run(check=True, args=["ungbfs", absolute_archive_1_path])

    # Extract archive 2 into temporary dir
    subprocess.run(check=True, args=["ungbfs", absolute_archive_2_path])

    print("TEMPDIR CONTENTS: {}".format(os.listdir(temp_dir)))
    # Pack back up
    all_files = os.listdir(temp_dir)

    # Have to do this in 2 steps, or we get "Invalid cross-device link"
    subprocess.run(check=True, args=["gbfs", "temp.gbfs"] + all_files)

    absolute_archive_out_path = os.path.abspath(archive_out_path)
    shutil.copy2("temp.gbfs", absolute_archive_out_path)

    # Go back to where we came from, because this function is frighteningly stateful.
    os.chdir(old_workdir)

