#!/usr/bin/env python3

import argparse
import os

# This script calls grit to convert the given image to Rust arrays (indirectly, by first converting to C as grit can't export to Rust)
import re
import subprocess

extract_asset_name_regex = re.compile(r"//\{\{BLOCK\((.*)\)")
find_array_regex = re.compile(r".* (.*)\[(\d+)\]")


def main():
    # Define CLI arguments
    parser = argparse.ArgumentParser(
        description="Converts an image format supported by GRIT to GBA-compatible Rust array"
    )
    parser.add_argument(
        "--grit-args",
        help="Arguments to pass through to grit. Note that no sanity checks are performed",
    )
    parser.add_argument("--output", help="Output Rust file", required=True)
    parser.add_argument("--input", help="Input bitmap file", required=True)
    args = parser.parse_args()

    c_file_path = call_grit(args.output, args.input, args.grit_args)
    convert(c_file_path, args.output)


def call_grit(output_path, input_path, grit_args):
    # Get path to C file to generate
    path, extension = os.path.splitext(output_path)
    if extension != ".rs":
        raise Exception("Output file has to end on .rs")
    c_file_path = "{}.c".format(path)

    # Run grit
    if grit_args == None:
        grit_extra = ""
    else:
        grit_extra = grit_args
    proc = subprocess.run(
        "grit {} -ftc -fh! -U32 --output {} {}".format(
            input_path, c_file_path, grit_extra
        ),
        shell=True,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return c_file_path


def convert(c_file_path, rust_file_path):
    with open(c_file_path, "r") as cf:
        with open(rust_file_path, "w+") as rf:
            rust_lines = []
            for line in cf:
                # Just copy line-by-line until we find array containing data
                if find_array_regex.match(line):
                    # Take the length and name of the array, and replace the C declaration with a Rust one
                    line_regex = find_array_regex.search(line)
                    # TODO: Pick proper type
                    rust_line = "pub(crate) const {}: [u32; {}]".format(
                        line_regex.group(1), line_regex.group(2))
                    rust_lines.append(rust_line)
                # If the line is any other, leave it alone
                else:
                    rust_lines.append(line)

            rf.writelines(rust_lines)
    os.remove(c_file_path)


if __name__ == "__main__":
    main()
