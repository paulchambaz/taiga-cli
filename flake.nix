{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
    let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages."${system}";
    fenixPkgs = fenix.packages."${system}";

    manifest = (pkgs.lib.importTOML ./Cargo.toml).package;

    buildPkgs = with pkgs; [
      scdoc
      pkg-config
    ];

    libPkgs = with pkgs; [
      openssl_3
    ];

    devPkgs = with pkgs; [
      just
      cargo-tarpaulin
      vhs
    ];

    makePkgConfigPath = libPkgs: pkgs.lib.concatStringsSep ":" (map (pkg: "${pkg.dev}/lib/pkgconfig") libPkgs);

    rust-toolchain = fenixPkgs.fromToolchainFile {
      file = ./rust-toolchain.toml;
      sha256 = "sha256-opUgs6ckUQCyDxcB9Wy51pqhd0MPGHUVbwRKKPGiwZU=";
    };

    rustPackage = {
      pname = manifest.name;
      version = manifest.version;
      src = self;

      cargoLock.lockFile = ./Cargo.lock;

      nativeBuildInputs = [ rust-toolchain ];
      buildInputs = buildPkgs ++ libPkgs;

      configurePhase = ''
        export PATH=${pkgs.lib.makeBinPath buildPkgs}:$PATH
        export PKG_CONFIG_PATH=${makePkgConfigPath libPkgs}:$PKG_CONFIG_PATH
        export HOME=$(mktemp -d)
      '';

      postInstall = ''
        mkdir -p $out/share/man/man1
        scdoc < taiga.1.scd > $out/share/man/man1/taiga.1
      '';

      meta = with pkgs.lib; {
        description = manifest.description;
        homepage = manifest.homepage;
        license = licenses.gpl3Plus;
        maintainers = with maintainers; [ paulchambaz ];
      };
    };

    shell = {

      shellHook = ''
        export PATH=${pkgs.lib.makeBinPath buildPkgs}:$PATH
        export PKG_CONFIG_PATH=${makePkgConfigPath libPkgs}:$PKG_CONFIG_PATH
      '';

      nativeBuildInputs = [ rust-toolchain ];
      buildInputs = buildPkgs ++ libPkgs ++ devPkgs;
    };
    in
  {
    packages."${system}".taiga-cli = pkgs.rustPlatform.buildRustPackage rustPackage;
    defaultPackage."${system}" = self.packages."${system}".taiga-cli;
    devShell."${system}" = pkgs.mkShell shell;
  };
}
