{
  description = "pray: package manager for the language placed before inference";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      version = cargoToml.workspace.package.version;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          pray-cli = pkgs.rustPlatform.buildRustPackage {
            pname = "pray-cli";
            inherit version;
            src = pkgs.lib.cleanSource self;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "-p" "pray-cli" ];
            cargoTestFlags = [ "-p" "pray-cli" ];
            meta = with pkgs.lib; {
              description = "Package manager for the language placed before inference";
              homepage = "https://pray.kisko.dev";
              license = licenses.mit;
              mainProgram = "pray";
              maintainers = [ ];
            };
          };
        in
        {
          inherit pray-cli;
          default = pray-cli;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/pray";
        };
      });

      overlays.default = final: prev: {
        pray-cli = self.packages.${prev.stdenv.hostPlatform.system}.pray-cli;
      };
    };
}
