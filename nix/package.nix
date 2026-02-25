{
  lib,
  rustPlatform,
  installShellFiles,
  pkg-config,
  wayland,
  libxkbcommon,
}:

rustPlatform.buildRustPackage {
  pname = "niri-sidebar";
  version = "0.1.0";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ./..)) (
      lib.fileset.unions [
        ../Cargo.toml
        ../Cargo.lock
        ../src
        ../default_config.toml
      ]
    );
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    installShellFiles
    pkg-config
  ];
  buildInputs = [
    wayland
    libxkbcommon
  ];

  postInstall = ''
    installShellCompletion --cmd niri-sidebar \
      --bash <($out/bin/niri-sidebar completions bash) \
      --zsh <($out/bin/niri-sidebar completions zsh) \
      --fish <($out/bin/niri-sidebar completions fish)
  '';

  meta = {
    description = "A sidebar for the Niri window manager";
    homepage = "https://github.com/Vigintillionn/niri-sidebar";
    license = lib.licenses.mit;
    mainProgram = "niri-sidebar";
    platforms = lib.platforms.linux;
  };
}
