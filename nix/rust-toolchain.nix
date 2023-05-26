{...}: {
  perSystem = {inputs', ...}: let
    cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);

    fenix-toolchain = inputs'.fenix.packages.toolchainOf {
      # Use the Rust version defined in `Cargo.toml`
      channel = cargoToml.package.rust-version;

      # THE rust-version HASH
      sha256 = "sha256-DzNEaW724O8/B8844tt5AVHmSjSQ3cmzlU4BP90oRlY=";
    };
  in rec {
    legacyPackages = {
      rust-toolchain = fenix-toolchain;
    };
  };
}
