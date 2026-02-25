{
  description = "A sidebar for the Niri window manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    flake-utils.lib.eachSystem supportedSystems (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          niri-sidebar = pkgs.callPackage ./nix/package.nix { };
          default = self.packages.${system}.niri-sidebar;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.niri-sidebar ];
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
          ];
        };
      }
    )
    // {
      overlays.default = _final: prev: {
        niri-sidebar = prev.callPackage ./nix/package.nix { };
      };

      homeModules.default = import ./nix/hm-module.nix self;
      homeModules.niri-sidebar = self.homeModules.default;
    };
}
