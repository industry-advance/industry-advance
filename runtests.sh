#!/usr/bin/env bash 

# This wrapper script is intended for use with "cargo xtest", not run standalone.
# It's needed because mGBA has no internal facility to exit successfully from within a ROM,
# see main.rs for details.

# Run mGBA headlessly, capture output and print at the same time
output=$( xvfb-run mgba -l -1 "$1" 2>&1 | tee /dev/tty)

# Check whether the process "failed successfully" (emitted the correct substring)
if [[ $output == *"Tests ran successfully, this panic is just here to quit mGBA"* ]]; then
    exit 0
fi

# For all other crashes, fail
exit 1