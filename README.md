# Industry-advance

A GBA game more-or-less loosely intended as a demake of Mindustry (Changes planned largely due to technical limitations).

## Build and run

The quickest way to get started under Linux/macOS is to use [https://nixos.org/nix/](Nix) as a dev environment. Just run `nix-shell` in the project directory.

Once set up, all that's needed is to run `cargo make run-qt` to build and start the game in mGBA. Alternatively, `cargo make debug-run` creates a debug build and launches it in mGBA, waiting for a GDB client to attach on port `2345`.