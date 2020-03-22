# Industry-advance

A GBA game more-or-less loosely intended as a demake of Mindustry (Changes planned largely due to technical limitations).

## Build and run

The quickest way to get started under Linux/macOS is to use [https://nixos.org/nix/](Nix) as a dev environment. Just run `nix-shell` in the project directory.

Once set up, all that's needed is to run `cargo make run-qt` to build and start the game in mGBA. Alternatively, `cargo make debug-run` creates a debug build and launches it in mGBA, waiting for a GDB client to attach on port `2345`.
`cargo make test` runs the tests.

## Further reading

* [A useful post on resource management for GBA games](https://www.gamasutra.com/view/feature/131491/gameboy_advance_resource_management.php?print=1)
* [Blog about developing an x86 OS in Rust, a lot of the posts apply to no_std dev in general](https://os.phil-opp.com)
* [Detailed guide to GBA programming](https://www.coranac.com/tonc)
