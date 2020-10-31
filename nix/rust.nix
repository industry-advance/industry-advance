{ sources ? import ./sources.nix }:

let
  pkgs =
    import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  channel = "nightly";
  date = "2020-10-25";
  targets = [ ];
  extensions =
    [ "rust-src" "clippy-preview" "rustfmt-preview" "rust-analyzer-preview" ];
  rustChannelOfTargetsAndExtensions = channel: date: targets: extensions:
    (pkgs.rustChannelOf { inherit channel date; }).rust.override {
      inherit targets extensions;
    };
  chan = rustChannelOfTargetsAndExtensions channel date targets extensions;
in chan
