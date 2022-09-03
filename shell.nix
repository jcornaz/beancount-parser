{ sources ? import ./nix/sources.nix
, pkgs ? import sources.nixpkgs { overlays = [(import sources.rust-overlay)]; }
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rust-bin.stable.latest.default
    clang
    mold
    niv
    rust-analyzer
    cargo-all-features
    cargo-deny
  ];
}

