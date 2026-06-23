
## Publishing to Nix

The project provides a Nix flake at the repository root. No separate publishing step is needed — users consume it directly from GitHub:

```bash
# Run directly
nix run github:vigintillionn/niri-sidebar

# Build locally
nix build github:vigintillionn/niri-sidebar

# Install via nix profile
nix profile install github:vigintillionn/niri-sidebar
```

### Installing Nix on Any Linux Distro

Nix is **not tied to NixOS** — it works as a package manager on any Linux distribution (Ubuntu, Arch, Fedora, etc.):

1. Install Nix: `sh <(curl -L https://nixos.org/nix/install)`
2. Enable flakes in `~/.config/nix/nix.conf`:
   ```
   experimental-features = nix-command flakes
   ```
3. Run the flake: `nix run github:vigintillionn/niri-sidebar`

### Flake Input

```nix
inputs = {
  dotstate.url = "github:vigintillionn/niri-sidebar";
};
```

### Using the Overlay

```nix
nixpkgs.overlays = [ inputs.niri-sidebar.overlays.default ];
environment.systemPackages = [ pkgs.niri-sidebar ];
```

### Using the Home Manager Module

```nix
imports = [ inputs.niri-sidebar.homeModules.default ];
programs.niri-sidebar.enable = true;
programs.niri-sidebar.settings = {
    geometry = {
        width = 400;
        height = 335;
        gap = 10;
    };
    margins = {
        top = 50;
        right = 10;
        left = 10;
        bottom = 10;
    };
    interaction = {
        position = "bottom";
        peek = 10;
        focus_peek = 50;
        sticky = true;
    };
};
```

### Non-Flake Usage

```bash
nix-build default.nix
```

### Updating Lockfiles (for maintainers)

After source changes, run this from the repo root to regenerate `Cargo.lock` and `flake.lock`:

```bash
./nix/rebuild.sh
```

Then commit:

```bash
git add Cargo.lock flake.lock
git commit -m "chore: update lockfiles"
```

> **Non-NixOS maintainers can still run this** — just install Nix on your distro first (see "Installing Nix on Any Linux Distro" above). The rebuild script only needs `cargo generate-lockfile`, `nix flake update`, and `nix build` — all available once Nix is installed.