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
        openssl
      ];

      devPkgs = with pkgs; [
        cargo
        rustfmt
        clippy
        rustc
        just
        cargo-tarpaulin
        vhs
      ];
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "taiga-cli";
        version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs;

        postInstall = ''
          mkdir -p $out/share/man/man1
          scdoc < taiga.1.scd > $out/share/man/man1/taiga.1
          install -Dm644 completions/_taiga $out/share/zsh/site-functions/_taiga
          install -Dm644 completions/taiga.bash $out/share/bash-completion/completions/taiga
          install -Dm644 completions/taiga.fish $out/share/fish/vendor_completions.d/taiga.fish
        '';

        meta = {
          description = "Cli tool for taiga.io inspired by taskwarrior";
          homepage = "https://github.com/paulchambaz/taiga-cli";
          license = pkgs.lib.licenses.gpl3Only;
          mainProgram = "taiga";
        };
      };

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs ++ devPkgs;
      };
    });
}
