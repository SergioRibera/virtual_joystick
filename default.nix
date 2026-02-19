let
  inherit
    (builtins)
    currentSystem
    fromJSON
    readFile
    ;
  getFlake = name:
    with (fromJSON (readFile ./flake.lock)).nodes.${name}.locked; {
      inherit rev;
      outPath = fetchTarball {
        url = "https://github.com/${owner}/${repo}/archive/${rev}.tar.gz";
        sha256 = narHash;
      };
    };
in
  {
    system ? currentSystem,
    pkgs ? import (getFlake "nixpkgs") {localSystem = {inherit system;};},
    lib ? pkgs.lib,
    fenix,
    stdenv ? pkgs.stdenv,
    ...
  }: let
    # fenix: rustup replacement for reproducible builds
    # toolchain = fenix.${system}.fromToolchainFile { dir = ./..; };
    toolchain = fenix.${system}.fromToolchainFile {
      file = ./rust-toolchain.toml;
      sha256 = "sha256-SBKjxhC6zHTu0SyJwxLlQHItzMzYZ71VCWQC2hOzpRY=";
    };

    buildInputs = with pkgs; [
      stdenv.cc.cc.lib
      alsa-lib
      udev
      libxkbcommon
      libxkbcommon.dev
      wayland
      wayland-protocols
      xorg.libX11
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      vulkan-loader
    ];

    deps = {
      nativeBuildInputs = with pkgs;
        [
          pkg-config
          autoPatchelfHook
        ]
        ++ lib.optionals stdenv.buildPlatform.isDarwin [
          pkgs.libiconv
        ]
        ++ lib.optionals stdenv.buildPlatform.isLinux [
          pkgs.libxkbcommon.dev
        ];
      runtimeDependencies = with pkgs;
        lib.optionals stdenv.isLinux [
          wayland
          libGL
          libxkbcommon
        ];
      inherit buildInputs;
    };

    commonArgs = targetName:
      deps
      // {
        src = lib.cleanSource ./.;
        doCheck = false;
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
        CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
        HOST_CC = "${stdenv.cc.nativePrefix}cc";
        pname = targetName;
        version = "0.1.0";
        cargoLock.lockFile = ./Cargo.lock;
        cargoBuildFlags = [ "-F" "inspect" "--example" targetName ];
      };

    invisibleApp = pkgs.rustPlatform.buildRustPackage (commonArgs "invisible");
    simpleApp    = pkgs.rustPlatform.buildRustPackage (commonArgs "simple");
    multipleApp  = pkgs.rustPlatform.buildRustPackage (commonArgs "multiple");
  in {
    # `nix run`
    apps = rec {
      simple    = { type = "app"; program = "${simpleApp}/bin/simple"; };
      multiple  = { type = "app"; program = "${multipleApp}/bin/multiple"; };
      invisible = { type = "app"; program = "${invisibleApp}/bin/invisible"; };
      default = multiple;
    };
    # `nix develop`
    devShells.default = pkgs.mkShell {
      packages = with pkgs;
        [
          toolchain
          pkg-config
          cargo-release
        ]
        ++ buildInputs;
      LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
    };
  }
