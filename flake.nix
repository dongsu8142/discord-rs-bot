{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };
  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { pkgs, lib, config, ... }:
        let
          inherit (lib.importTOML (inputs.self + "/Cargo.toml")) package;
        in
        {
          packages = {
            default = pkgs.rustPlatform.buildRustPackage {
              pname = package.name;
              inherit (package) version;
              src = inputs.self;
              cargoLock = {
                lockFile = (inputs.self + "/Cargo.lock");
                allowBuiltinFetchGit = true;
              };
              buildInputs = with pkgs; [
                openssl
                libopus
              ];
              nativeBuildInputs = with pkgs; [
                pkg-config
                cmake
              ];
            };
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              openssl
              libopus
              yt-dlp
            ];
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              pkg-config
              cmake
            ];
          };

          apps = {
            default = {
              program = "${config.packages.default}/bin/${package.name}";
              type = "app";
            };
          };
        };
    };
}
