{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      buildPkgs = with pkgs; [
        pkg-config
        scdoc
      ];

      libPkgs = with pkgs; [
        openssl_3
      ];

      devPkgs = with pkgs; [
        cargo
        rustfmt
        rustc
        just
        cargo-tarpaulin
        vhs
      ];
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "taiga-cli";
        version = "1.1.3";
        src = ./.;
        cargoHash = "sha256-dKqCEJU01AcXKeYq8psMFBzVzYzbxY7Zxfnq5rFc0MI=";

        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs;

        postInstall = ''
          mkdir -p $out/share/man/man1
          scdoc < taiga.1.scd > $out/share/man/man1/taiga.1
        '';
      };

      devShell = pkgs.mkShell {
        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs ++ devPkgs;
      };
    });
}
