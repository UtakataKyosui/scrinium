{
  description = "Rust Development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
  let
    system = "aarch64-linux";

    overlays = [
      (import rust-overlay)
    ];

    pkgs = import nixpkgs {
      inherit system overlays;
    };
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        (rust-bin.stable.latest.default)

        rust-analyzer

        git
        jujutsu
        just

        eza
        fd
        ripgrep
        bat

        jq
        yq-go

        openssl
        pkg-config

        typos

        cargo-nextest
        cargo-watch
        cargo-edit
        cargo-deny
        cargo-audit

        bacon
      ];
    };
  };
}