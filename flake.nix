{
  description = "VirtualJoystick Bevy lib";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  } @ inputs:
    # Iterate over Arm, x86 for MacOs 🍎 and Linux 🐧
    flake-utils.lib.eachSystem (flake-utils.lib.defaultSystems) (
      system: let
        bevyLibBundle = import ./. {
          inherit system;
          pkgs = nixpkgs.legacyPackages.${system};
          fenix = inputs.fenix.packages;
        };
      in {
        inherit (bevyLibBundle) apps devShells;
      }
    );
}
