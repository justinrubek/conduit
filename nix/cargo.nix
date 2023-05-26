{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    inputs',
    self',
    lib,
    ...
  }: let
    inherit (self'.packages) rocksdb;
    inherit (self'.legacyPackages) rust-toolchain llvmPackages;

    # packages required for building the rust packages
    extraPackages = [
      pkgs.pkg-config
      pkgs.rustPlatform.bindgenHook
    ];
    withExtraPackages = base: base ++ extraPackages;

    craneLib = inputs.crane.lib.${system}.overrideToolchain rust-toolchain.toolchain;

    common-build-args = rec {
      src = inputs.nix-filter.lib {
        root = ../.;
        include = [
          "src"
          "Cargo.toml"
          "Cargo.lock"
        ];
      };

      pname = "conduit";

      nativeBuildInputs = withExtraPackages [];

      BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.getVersion pkgs.clang}/include";
      LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
      # ROCKSDB_INCLUDE_DIR = "${rocksdb}/include";
      # ROCKSDB_LIB_DIR = "${rocksdb}/lib";
    };

    deps-only = craneLib.buildDepsOnly ({} // common-build-args);

    packages = {
      conduit = craneLib.buildPackage ({
          pname = "conduit";
          cargoArtifacts = deps-only;
          meta.mainProgram = "conduit";
        }
        // common-build-args);
      default = packages.conduit;

      cargo-doc = craneLib.cargoDoc ({
          cargoArtifacts = deps-only;
        }
        // common-build-args);
    };

    checks = {
      clippy = craneLib.cargoClippy ({
          cargoArtifacts = deps-only;
          cargoClippyExtraArgs = "--all-features -- --deny warnings";
        }
        // common-build-args);

      rust-fmt = craneLib.cargoFmt ({
          inherit (common-build-args) src;
        }
        // common-build-args);

      rust-tests = craneLib.cargoNextest ({
          cargoArtifacts = deps-only;
          partitions = 1;
          partitionType = "count";
        }
        // common-build-args);
    };
  in {
    inherit packages checks;

    legacyPackages = {
      cargoPackages = extraPackages;
    };
  };
}
