{
  description = "A flake for the pre-compiled wasm-pack binary (v0.14.0)";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: 
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      
      # Using a standard helper pattern to avoid deprecated accessors
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f pkgsFor.${system});
pkgsFor = nixpkgs.lib.genAttrs supportedSystems (system: import nixpkgs {
  inherit system;
 # config.allowUnfree = true; # If you ever need it
});
    in
    {
      packages = forAllSystems (pkgs: {
        wasm-pack = pkgs.callPackage ./default.nix { };
        default = self.packages.${pkgs.stdenv.hostPlatform.system}.wasm-pack;
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = [ self.packages.${pkgs.stdenv.hostPlatform.system}.wasm-pack ];
        };
      });

      nixosModules.default = { config, lib, pkgs, ... }: 
      let
        cfg = config.programs.wasm-pack;
      in {
        options.programs.wasm-pack.enable = lib.mkEnableOption "wasm-pack";
        config = lib.mkIf cfg.enable {
          # Use the platform-correct system string here too
          environment.systemPackages = [ self.packages.${pkgs.stdenv.hostPlatform.system}.default ];
        };
      };

      homeManagerModules.default = { config, lib, pkgs, ... }:
      let
        cfg = config.programs.wasm-pack;
      in {
        options.programs.wasm-pack.enable = lib.mkEnableOption "wasm-pack";
        config = lib.mkIf cfg.enable {
          home.packages = [ self.packages.${pkgs.stdenv.hostPlatform.system}.default ];
          home.sessionVariables.WASM_PACK_CACHE = "${config.xdg.cacheHome}/wasm-pack";
        };
      };
    };
}
