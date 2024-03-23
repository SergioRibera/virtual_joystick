{
  description = "VirtualJoystick Bevy lib";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cranix.url = "github:Lemin-n/cranix";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
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
          crane = inputs.crane.lib;
          cranix = inputs.cranix.lib;
          fenix = inputs.fenix.packages;
        };
      in {
        inherit (bevyLibBundle) apps devShells;
      }
    );
}
