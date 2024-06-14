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
        just
        cargo-tarpaulin
        vhs
      ];
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "taiga-cli";
        version = "1.0.0";
        src = ./.;
        cargoHash = "sha256-Li4pxu1JnIfuOGy51/FrFj5DTZ3oWuzg647qYgWyGmk=";

        buildInputs = libPkgs ++ buildPkgs;

        configurePhase = ''
          export PATH=${pkgs.lib.makeBinPath buildPkgs}:$PATH
          export PKG_CONFIG_PATH="${pkgs.openssl_3.dev}/lib/pkgconfig"
        '';

        postInstall = ''
          mkdir -p $out/share/man/man1
          scdoc < taiga.1.scd > $out/share/man/man1/taiga.1
        '';
      };
      devShell = pkgs.mkShell {
        buildInputs = libPkgs ++ buildPkgs ++ devPkgs;

        shellHook = ''
          export PKG_CONFIG_PATH="${pkgs.openssl_3.dev}/lib/pkgconfig"
        '';
      };
    });
}
