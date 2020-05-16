# Industry-advance

A GBA game more-or-less loosely intended as a demake of Mindustry (Changes planned largely due to technical limitations).

## Build and run

The quickest way to get started under Linux/macOS is to use [https://nixos.org/nix/](Nix) as a dev environment. Just run `nix-shell` in the project directory.

Once set up, all that's needed is to run `git submodule init && git submodule update && cargo make assets && cargo make run-qt` to clone the Mindustry submodule, build assets, build the game and start it in mGBA. Alternatively, `cargo make assets && cargo make debug-run` creates a debug build and launches it in mGBA, waiting for a GDB client to attach on port `2345` (you should strongly consider using our VScode debug config, as all the annoying setup has already been done for you there).
`cargo make assets && cargo make test` runs the tests.

*NOTE:* If you change the Mindustry assets/the asset generation script you have to run `cargo make assets` manually. The process is manual because rebuilding them takes a long time.

## HW resource map

When developing the game, please keep in mind which subsystems use which
resources (and expect exclusive access to them). Otherwise, fun debugging will ensue.

* `IWRAM:` Entirely used by static variables and the stack
* `EWRAM:` Entirely used by the global allocator (heap)
* `OAM`: Entirely managed by the HW sprite allocator
* `Sprite palette:` Entirely managed by the HW sprite allocator
* `Background palette (slots 0,1):` Used by the text engine
* `Background palette (slots 2-255):` Entirely managed by the background system
* `Charblocks 0, 1:` Entirely managed by the background system
* `Screenblocks 8-11:` Entirely managed by the background system
* `Charblock 2:` Entirely managed by the text engine
* `Screenblock 24:` Entirely managed by the text engine
* `Charblock 3:` Unusable, as it overlaps the screenblocks we use

## Further reading

* [A useful post on resource management for GBA games](https://www.gamasutra.com/view/feature/131491/gameboy_advance_resource_management.php?print=1)
* [Blog about developing an x86 OS in Rust, a lot of the posts apply to no_std dev in general](https://os.phil-opp.com)
* [Detailed guide to GBA programming](https://www.coranac.com/tonc)
* [Text engine which heavily inspired the design of ours](https://www.coranac.com/tonc/text/text.htm)
* [Font we use](https://int10h.org/oldschool-pc-fonts/fontlist/#ibmcga)
