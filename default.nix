{
  lib,
  stdenv,
  fetchurl,
}:
let
  sources = import ./sources.nix;
  system = stdenv.hostPlatform.system;

  # Access 'assets' attribute created by the Rust script
  asset = sources.assets.${system} or (throw "Unsupported system: ${system}");
in
stdenv.mkDerivation {
  pname = "wasm-pack";
  version = sources.version; # Uses the version from Rust/GitHub API

  src = fetchurl {
    url = asset.url;   # Use the full URL from sources.nix
    hash = asset.hash; # Uses the "sha256:..." hex format
  };

  sourceRoot = ".";

  installPhase = ''
    runHook preInstall

    mkdir -p $out/bin
    # Search for the binary in the unpacked source
    find . -maxdepth 2 -name "wasm-pack" -type f -exec cp {} $out/bin/ \;
    chmod +x $out/bin/wasm-pack

    runHook postInstall
  '';

    meta = with lib; {
      description = "Your favorite rust -> wasm workflow tool";
      homepage = "https://github.com/rustwasm/wasm-pack";
      # This maps the strings ["asl20" "mit"] to [lib.licenses.asl20 lib.licenses.mit]
      # The or k ensures that if a key is missing, Nix just returns the string instead of crashing the whole evaluation.
      license = map (k: licenses.${k} or k) sources.licenseKeys;
      platforms = builtins.attrNames sources.assets;
      mainProgram = "wasm-pack";
    };
}
