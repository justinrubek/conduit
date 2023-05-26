{inputs, ...}: {
  perSystem = {pkgs, ...}: let
    llvmPackages = pkgs.llvmPackages_11;
    clang = pkgs.clang_11;
  in {
    packages = {
      inherit clang;

      rocksdb = pkgs.rocksdb_6_23;
    };

    legacyPackages = {
      inherit llvmPackages;

      rocksPackages = [
        clang
        llvmPackages.bintools
        llvmPackages.libclang
        # pkgs.rustPlatform.bindgenHook
      ];
    };
  };
}
