{
  description = "A modal interface for Jujutsu.";

  # Note: For faster builds, users can add these caches to their Nix configuration:
  # - https://crane.cachix.org
  # - https://nix-community.cachix.org

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    inputs@{
      flake-parts,
      nixpkgs,
      gitignore,
      rust-overlay,
      crane,
      advisory-db,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          self',
          system,
          ...
        }:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource (gitignore.lib.gitignoreSource ./.);

          commonArgs = {
            inherit src;
            strictDeps = true;
            buildInputs =
              [ ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.libiconv
              ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          jjj = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;
              checkPhase = ''
                export INSTA_UPDATE=no
                ${craneLib.buildPackage.checkPhase or ""}
              '';
            }
          );
        in
        {
          _module.args.pkgs = pkgs;

          checks = {
            inherit jjj;

            fmt = craneLib.cargoFmt { inherit src; };

            toml-fmt = craneLib.taploFmt {
              src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            };

            clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

            doc = craneLib.cargoDoc (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );

            audit = craneLib.cargoAudit {
              inherit src advisory-db;
            };

            # TODO: define acceptable licenses with cargo-deny
            # licenses = craneLib.cargoDeny {
            #   inherit src;
            # };

            # TODO: make snapshot tests fail properly when running checks
            nextest = craneLib.cargoNextest (
              commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
                checkPhase = ''
                  export INSTA_UPDATE=no
                  ${craneLib.cargoNextest.checkPhase or ""}
                '';
                nativeBuildInputs = [ pkgs.cargo-insta ];
              }
            );
          };

          packages.default = jjj;

          apps.default = {
            type = "app";
            program = "${jjj}/bin/jjj";
            meta.description = "A modal interface for Jujutsu.";
          };

          devShells.default = craneLib.devShell {
            checks = self'.checks;
          };

          formatter = pkgs.nixfmt-rfc-style;
        };
    };
}
