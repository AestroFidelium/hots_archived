{
  description = "Global Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            gcc
            pkg-config
            openssl
            cargo-watch
            cargo-edit
            
            # Wayland support
            wayland
            libxkbcommon
            
            # X11 support
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            
            # OpenGL/Mesa support (CRITICAL for glium/glutin)
            libGL
            libGLU
            mesa
            
            # EGL support
            libglvnd
            egl-wayland
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.wayland
            pkgs.libxkbcommon
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXrandr
            pkgs.xorg.libXi
            pkgs.libGL
            pkgs.libGLU
            pkgs.mesa
            pkgs.libglvnd
            pkgs.egl-wayland
          ];
          
          shellHook = ''
            echo "🦀 Global Rust development environment"
            echo "Rust: $(rustc --version)"
            echo "Cargo: $(cargo --version)"
            export LIBGL_ALWAYS_SOFTWARE=0
          '';
        };
      }
    );
}