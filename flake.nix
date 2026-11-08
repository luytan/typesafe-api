{
  description = "Rust development shell template";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      fenixpkgs = fenix.packages.${system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-p8h3Sl/YRByZfZTAKXdsvF6xEenXKrXSVvpphmZENH4=";
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          fenixpkgs
          pkgs.cargo-bootimage
        ];
        RUST_SRC_PATH = "${fenixpkgs}/lib/rustlib/src/rust/library";
      };
    };
}
