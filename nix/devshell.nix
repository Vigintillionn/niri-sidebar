{
  pkgs,
  niri-sidebar,
}:
pkgs.mkShell {
  inputsFrom = [ niri-sidebar ];

  nativeBuildInputs = with pkgs; [
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
  ];

  shellHook = ''
    echo " niri-sidebar dev-shell | 'cargo build' to build | 'cargo test' to test"
  '';
}
