# niri-sidebar

A lightweight, external sidebar manager for the [Niri](https://github.com/YaLTeR/niri) window manager.

`niri-sidebar` allows you to toggle any window into a "floating sidebar" stack on the right side of your screen. It automatically handles resizing, positioning, and stacking, keeping your main workspace clean while keeping utility apps (terminals, music players, chats) accessible.

https://github.com/user-attachments/assets/46f51b18-d85b-4d79-9c44-63e63649707a

## Features

- **Toggle Windows:** Instantly move the focused window into the sidebar stack.
- **Keyboard-Driven Focus:** Cycle through sidebar windows with `focus-next`/`focus-prev`, then pull one out or return to tiling — no mouse needed.
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

Add the following bindings to your niri `config.kdl` file.

**Important:** These examples assume you installed the tool to `~/.local/bin`. If you installed it elsewhere, update the paths accordingly.

```kdl
binds {
    // Toggle the focused window into/out of the sidebar
    Mod+S { spawn-sh "~/.local/bin/niri-sidebar toggle-window"; }

    // Toggle sidebar visibility (hide/show)
    Mod+Shift+S { spawn-sh "~/.local/bin/niri-sidebar toggle-visibility"; }

    // Flip the order of the sidebar
    Mod+Ctrl+S { spawn-sh "~/.local/bin/niri-sidebar flip"; }

    // Focus the next sidebar window
    Mod+J { spawn-sh "~/.local/bin/niri-sidebar focus-next"; }

    // Focus the previous sidebar window
    Mod+K { spawn-sh "~/.local/bin/niri-sidebar focus-prev"; }

    // Force reorder (useful if something gets misaligned manually)
    Mod+Alt+R { spawn-sh "~/.local/bin/niri-sidebar reorder"; }
}
```

### Seamless tiling/sidebar navigation

When a sidebar window is focused, niri's native `focus-column-left`/`focus-column-right` won't return you to the tiling layout. To make `Mod+H`/`Mod+L` context-aware — returning to tiling when on a sidebar window, or moving between columns when already in tiling — create a helper script at `~/.local/bin/niri-focus-column`:

```sh
#!/bin/sh
if niri msg focused-window 2>/dev/null | grep -q "Is floating: yes"; then
    niri msg action focus-tiling
else
    niri msg action "focus-column-$1"
fi
```

Then bind it in place of the native actions:

```kdl
binds {
    Mod+H { spawn-sh "~/.local/bin/niri-focus-column left"; }
    Mod+L { spawn-sh "~/.local/bin/niri-focus-column right"; }
}
```

### Startup daemon

In order for your sidebar to stay consistent and gap free, you want to add the following to your startup scripts:

```kdl
spawn-at-startup "~/.local/bin/niri-sidebar" "listen"
```

This will spawn a daemon to listen for window close events and reorder the sidebar if the closed window was part of it.

### Window rules

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

- **Adding:** Focus a tiling window and press `Mod+S` to snap it into the sidebar.
- **Browsing the sidebar:** Press `Mod+J` to enter at the bottom of the sidebar, `Mod+K` to enter at the top. Subsequent presses cycle through the stack with wrap-around.
- **Pulling a window out:** Press `Mod+S` on a focused sidebar window to return it to tiling. Focus automatically moves back to the tiling layout.
- **Returning to tiling:** Press `Mod+H` (with the helper script above) to leave the sidebar and return to your tiling columns without pulling anything out.
- **Hiding:** Press `Mod+Shift+S` to tuck the sidebar away. It will stick out slightly (configured by `peek`) so you know it's there.

**Typical session:** Tiling as usual with `Mod+H`/`Mod+L` → `Mod+S` to push a window to the sidebar → `Mod+J`/`Mod+K` to browse → `Mod+S` to pull one back out, or `Mod+H` to return to tiling empty-handed.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
