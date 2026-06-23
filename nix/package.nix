{
  lib,
  rustPlatform,
}:
let
  inherit (builtins) readFile;
  cargoToml = builtins.fromTOML (readFile ../Cargo.toml);
  version = cargoToml.package.version;
in
rustPlatform.buildRustPackage {
  pname = "niri-sidebar";
  inherit version;

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  doCheck = true;

  meta = with lib; {
    description = "A lightweight, external sidebar manager for the Niri window manager";
    homepage = "https://github.com/Vigintillionn/niri-sidebar";
    license = licenses.mit;
    maintainers = with maintainers; [ ];
    platforms = platforms.linux;
    mainProgram = "niri-sidebar";
    sourceProvenance = with sourceTypes; [ binaryNativeCode ];
  };
}
