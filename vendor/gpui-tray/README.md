# gpui-tray

Native system tray integration built specifically for GPUI. Tray menu entries
are ordinary `gpui::MenuItem`s and clicks dispatch ordinary GPUI actions.

The crate has three native backends:

- macOS: AppKit through `objc2`
- Windows: `Shell_NotifyIconW` plus a hidden top-level window and Win32 menus
- Linux: StatusNotifierItem and DBusMenu through `zbus` (no GTK)

It does not start a second application event loop. Native callbacks send small,
platform-neutral events to a task on GPUI's foreground executor.

## Run the example

From a graphical desktop session, run:

```sh
cargo run --example tray
```

The example has no application window. Use its tray menu to change checked and
disabled state, increment the counter, and quit. Its icon is generated in code,
so no external assets are required.

```rust,ignore
use gpui::{App, Menu, MenuItem, actions};
use gpui_tray::{Icon, Tray};

actions!(tray_example, [Open, ToggleMode, Quit]);

fn build_tray(cx: &mut App, icon: Icon) -> gpui_tray::Result<Tray> {
    Tray::builder()
        .icon(icon)
        .title("My App")
        .tooltip("My App")
        .menu(|_cx| {
            vec![
                MenuItem::action("Open", Open),
                MenuItem::submenu(
                    Menu::new("View")
                        .items([MenuItem::action("Toggle mode", ToggleMode)]),
                ),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ]
        })
        .build(cx)
}
```

Call `Tray::refresh_menu` after state changes outside tray actions. The menu is
automatically rebuilt after a tray action is dispatched, and checked/disabled
state produced by synchronous action handlers updates immediately.

`Tray::close` removes the native item deterministically and is safe to call more
than once. Dropping the last clone performs the same cleanup on a best-effort
basis.

Checked and disabled menu state is provided directly through the current GPUI
menu API.
