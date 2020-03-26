# Industry-advance

A GBA game more-or-less loosely intended as a demake of Mindustry (Changes planned largely due to technical limitations).

## Build and run

The quickest way to get started under Linux/macOS is to use [https://nixos.org/nix/](Nix) as a dev environment. Just run `nix-shell` in the project directory.

Once set up, all that's needed is to run `cargo make assets && cargo make run-qt` to build assets, build the game and start it in mGBA. Alternatively, `cargo make assets && cargo make debug-run` creates a debug build and launches it in mGBA, waiting for a GDB client to attach on port `2345`.
`cargo make assets && cargo make test` runs the tests.

*NOTE:* If you change the Mindustry assets/the asset generation script you have to run `cargo make assets` manually. The process is manual because rebuilding them takes a long time.

## Update CI build environment

The docker container for CI is built with nix as well in order to ensure reproducibility between dev and test environments.

To update the container, run `nix-build docker.nix`, then load the generated image tarball with `docker load -i /path/to/tarball` and push it to the image repository.

## Further reading

* [A useful post on resource management for GBA games](https://www.gamasutra.com/view/feature/131491/gameboy_advance_resource_management.php?print=1)
* [Blog about developing an x86 OS in Rust, a lot of the posts apply to no_std dev in general](https://os.phil-opp.com)
* [Detailed guide to GBA programming](https://www.coranac.com/tonc)
