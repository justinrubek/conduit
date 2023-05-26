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
    inherit (self'.legacyPackages) rust-toolchain cargoPackages rocksPackages llvmPackages;

    devTools = [
      # rust tooling
      rust-toolchain.toolchain
      pkgs.cargo-audit
      pkgs.cargo-udeps
      pkgs.bacon
    ];
  in {
    devShells = {
      default = pkgs.mkShell rec {
        packages = devTools ++ cargoPackages ++ rocksPackages;

        BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${llvmPackages.libclang.lib}/lib/clang/${lib.getVersion pkgs.clang}/include";
        LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
        RUST_SRC_PATH = "${rust-toolchain.toolchain}/lib/rustlib/src/rust/src";
        # ROCKSDB_INCLUDE_DIR = "${rocksdb}/include";
        # ROCKSDB_LIB_DIR = "${rocksdb}/lib";
      };
    };
  };
}
