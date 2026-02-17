{
  stdenv,
  fetchurl,
}:
let
  sources = import ./sources.nix;
  system = stdenv.hostPlatform.system;

  # Specify which repo from sources.nix you want to package here
  repoData = sources."wasm-pack" or (throw "wasm-pack not found in sources.nix");

  # Access assets for the current system from that specific repo
  asset = repoData.assets.${system} or (throw "Unsupported system: ${system} for wasm-pack");
in
stdenv.mkDerivation {
  pname = "wasm-pack";
  version = repoData.version; 

  src = fetchurl {
    inherit (asset) url hash;
  };

  # Since it's a pre-compiled binary, we just install it
  installPhase = ''
    install -m755 -D $src $out/bin/wasm-pack
  '';
}
