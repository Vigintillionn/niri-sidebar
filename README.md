# niri-sidebar

A lightweight, external sidebar manager for the [Niri](https://github.com/YaLTeR/niri) window manager.

`niri-sidebar` allows you to toggle any window into a "floating sidebar" stack on the right side of your screen. It automatically handles resizing, positioning, and stacking, keeping your main workspace clean while keeping utility apps (terminals, music players, chats) accessible.

## Features

- **Toggle Windows:** Instantly move the focused window into the sidebar stack.
- **Auto-Stacking:** Windows automatically stack vertically with a configurable gap.
- **Smart Close:** Closing a sidebar window automatically reorders the remaining windows to fill the gap.
- **Flip & Hide:** Flip the stack to the other side of the screen or hide it completely (peeking mode).
- **State Persistence:** Remembers your sidebar windows and their original sizes even if you restart the tool.

## Installation

### Option 1: Download Binary (Recommended)

1.  Go to the [Releases](https://github.com/Vigintillionn/niri-sidebar/releases) page.
2.  Download the `niri-sidebar` binary.
3.  Make it executable and move it to your path:

```bash
chmod +x niri-sidebar
sudo mv niri-sidebar /usr/local/bin/
# OR
mv niri-sidebar ~/.local/bin/
```

### Option 2: Build from Source

```bash
git clone https://github.com/Vigintillionn/niri-sidebar
cd niri-sidebar
cargo build --release
cp target/release/niri-sidebar ~/.local/bin/
```

## Niri configuration

Add the following bindings to your niri.kdl config file.

**Important:** These examples assume you installed the tool to `~/.local/bin`. If you installed it elsewhere, update the paths accordingly.

**Note:** It is highly recommended to replace your default "Close Window" bind with niri-sidebar close. This command checks if the window is in the sidebar first. If it is, it cleanly removes it and reorders the stack. If it's a normal window, it just closes it like normal.

```kdl
binds {
    // Toggle the focused window into/out of the sidebar
    Mod+S { spawn-sh "~/.local/bin/niri-sidebar toggle-window"; }

    // Toggle sidebar visibility (hide/show)
    Mod+Shift+S { spawn-sh "~/.local/bin/niri-sidebar toggle-visibility"; }

    // Flip the order of the sidebar
    Mod+Ctrl+S { spawn-sh "~/.local/bin/niri-sidebar flip"; }

    // Force reorder (useful if something gets misaligned manually)
    Mod+Alt+R { spawn-sh "~/.local/bin/niri-sidebar reorder"; }

    // RECOMMENDED: Replacement Close Bind
    // Keeps the sidebar gap-free when closing a sidebar window.
    Mod+Q { spawn-sh "~/.local/bin/niri-sidebar close"; }
}
```

Some applications enforce a minimum window size that is larger than your sidebar configuration, which can cause windows to overlap or look broken. Add this rule to force them to respect the sidebar size:

```kdl
window-rule {
    match is-floating=true
    min-width 100
    min-height 100
}
```

## Configuration

Run `niri-sidebar init` to generate a `config.toml` file located at `~/.config/niri-sidebar`.

#### Default Config

```toml
# niri-sidebar configuration

# Width of the sidebar in pixels
sidebar_width = 400

# Height of the sidebar windows
sidebar_height = 335

# Space from the top/bottom of the screen
offset_top = 50

# Space from the right edge of the screen
offset_right = 10

# Gap between windows in the stack
gap = 10

# Width of windows when sidebar is hidden in pixels
peek = 10
```

## Workflow tips

- **Adding/Removing:** Press `Mod+S` on any window to snap it into the sidebar. Press it again to return it to your normal tiling layout.
- **Hiding:** Press `Mod+Shift+S` to tuck the sidebar away. It will stick out slightly (configured by peek) so you know it's there.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
