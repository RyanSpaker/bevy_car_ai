{
  description = "Rust VS Code Dev Environment";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    nix-vscode-extensions.url = "github:nix-community/nix-vscode-extensions";
  };
  outputs = { self, nixpkgs, flake-utils, fenix, nix-vscode-extensions, ...}:
    flake-utils.lib.eachDefaultSystem (
      system: 
      let
        pkgs = import nixpkgs {inherit system; config.allowUnfree = true; };
        extensions = nix-vscode-extensions.extensions.${system}.vscode-marketplace;
        f = fenix.packages.${system};
      in
      {
        devShells.default = with pkgs; 
        let
          rust-toolchain = (f.fromToolchainName { 
            name = (lib.importTOML ./rust-toolchain.toml).toolchain.channel; 
            sha256 = "sha256-tMb9kczlM4TBLx4r5Z3Xw84x33QzD/YI6JMjGdl+oC8=";
          });
        in
        mkShell rec{
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            (f.combine [
              (f.latest.withComponents [
                "cargo"
                "clippy"
                "rust-src"
                "rustc"
                "rustfmt"
              ])
              f.targets.wasm32-unknown-unknown.latest.rust-std
            ])
            cargo-udeps
            git
            (rustPlatform.buildRustPackage rec {
              pname = "wasm-server-runner";
              version = "0.6.3";

              src = fetchCrate {
                inherit pname version;
                hash = "sha256-4NuvNvUHZ7n0QP42J9tuf1wqBe9f/R6iJAGeuno9qtg=";
              };

              cargoHash = "sha256-aq4hrZPRgKdRNvMrE9Lhy3AD7lXb/UocNUNpeNZz3cM=";
              cargoDepsName = pname;
            })
            (vscode-with-extensions.override {
              vscode = vscodium;
              vscodeExtensions = with extensions; [
                rust-lang.rust-analyzer
                jnoortheen.nix-ide
                arrterian.nix-env-selector
                mkhl.direnv
                polymeilex.wgsl
              ];
            })
            udev udev.dev alsa-lib lutris
            vulkan-tools vulkan-headers vulkan-loader vulkan-validation-layers
            xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
            libxkbcommon wayland # To use the wayland feature
            rustc.llvmPackages.clang
            rustc.llvmPackages.bintools
            fish
            (wrapBintoolsWith { bintools = mold; })
          ];
          LIBCLANG_PATH = lib.makeLibraryPath [ rustc.llvmPackages.libclang.lib ];
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          RUST_SRC_PATH = "${f.latest.rust-src}/lib/rustlib/src/rust/library";
          PATH = "${f.latest.cargo}/bin";
        };
      }
    );
}
