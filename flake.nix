{
  description = "offline rhizometrack app, level up your skills through spending time with them";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        nativeBuildInputs = with pkgs; [ pkg-config ];


        buildInputs = with pkgs; [
          freetype
          fontconfig
          libxkbcommon
          vulkan-loader
          wayland
          wayland-protocols
          libGL
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];

        libPath = pkgs.lib.makeLibraryPath buildInputs;

      in {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            cargo-watch
            sqlite
          ];

          # Exposes the libraries to cargo run during development
          LD_LIBRARY_PATH = libPath;

        };

        packages.rhizometrack = pkgs.rustPlatform.buildRustPackage {
          pname = "rhizometrack";
          version = "1.0.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;

          # Adds the dynamically loaded libraries back to the built binary's rpath
          # so they aren't stripped by Nix's patchelf during the fixup phase.
          postFixup = ''
            patchelf --add-rpath ${libPath} $out/bin/rhizometrack
          '';
        };
postInstall = ''
      wrapProgram $out/bin/rhizometrack \
        --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath (with pkgs; [
          wayland
          libxkbcommon
          libGL
          vulkan-loader
        ])}"
    '';
        defaultPackage = self.packages.${system}.rhizometrack;
        defaultApp = self.apps.${system}.rhizometrack;

        apps.rhizometrack = {
          type = "app";
          program = "${self.packages.${system}.rhizometrack}/bin/rhizometrack";
        };
      });
}
