{
  description = "Rust Development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nix-claude-code.url = "github:ryoppippi/nix-claude-code";
  };

  outputs = { self, nixpkgs, rust-overlay, nix-claude-code}:
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

        direnv
        nix-direnv
        
        rust-analyzer

        git
        jujutsu
        just

        eza
        fd
        ripgrep
        bat
        atuin

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
        nix-claude-code.packages.${system}.default

        gh
      ];
    };
  };
}