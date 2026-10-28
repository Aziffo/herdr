# Agent highlights in the Aziffo fork

Based on the official Herdr v0.9.5 stable release.

Right-click an agent in the active machine sidebar, choose **Highlight**, then
choose a color or **None**. The menu captures the clicked pane, so identically
named agents in the same tab have independent colors. Opening or using the menu
does not focus a different pane. Agent rows never inherit a tab/workspace color.
The indicator uses the current theme and appears in expanded and collapsed lists.
Right-clicking a terminal pane also offers Highlight. Workspace and tab menus
keep their explicit workspace/tab scope.

This fork adds native client UI to the existing
[aziffo.highlight](https://github.com/Aziffo/herdr-highlight) plugin (0.2.0+).
Install it on every endpoint:

```sh
herdr plugin install Aziffo/herdr-highlight --ref main --yes
```

The plugin owns SQLite persistence, movement/close hooks, and startup recovery.
The fork invokes its existing public `plugin.action.invoke` method with explicit
workspace, tab, and focused-pane context. The endpoint advertises that method
additively; published binary codecs and existing JSON method shapes are unchanged.
A server without the advertised method reports an unavailable action.

Both client and endpoint servers must run this build. Selecting another machine
before using its context menu is required; inactive-machine rows cannot accidentally
apply a color to a colliding pane ID on the active machine.

Build with `just build` (Rust and Zig 0.16 required), and use the project checks
with `just check`. `just test-one highlight` covers duplicate names, mouse and
keyboard selection, None, cancellation, tiny menus, rendering, and endpoint IDs.
Ordinary official binary updates replace this fork; retain or rebuild this patch
when moving to a newer upstream version.

## Verification on 2026-10-28

- Rust 1.96.1: formatting and clippy with warnings denied passed.
- All 4,035 non-ignored Rust tests passed (15 manual tests skipped).
- Maintenance, UI architecture, and integration asset tests passed.
- Real PTY/TUI mouse test: two `simpleLoginlocal` agents in one tab; right-click
  the second, select Blue, verify the first is unchanged, then select None.
  Focus remains unchanged throughout.
- The plugin's isolated real-server integration passes against this binary,
  including independent colors, restore, pane moves, and close cleanup.
- Fixed 40x50 client rendering, 500 debug-build iterations: 1 agent takes
  54.86 ms without a marker / 57.89 ms with it; 15 agents take 788.62 ms /
  802.77 ms. These are observational timings, not performance assertions.
- The full `just check` reaches Windows cross-lint, which cannot run on the
  deployment host without its Windows SDK. Linux checks above passed; Windows
  binary deployment is not claimed.
- The headless hangup test now waits for runtime readiness, because its API
  socket becomes available before shutdown handlers finish initialization.

Deployed binary SHA-256:
`3ba761e6ed528b6ce93a8a77aee252ac67361f78fd113b969057ab6ca5108048`.
Live handoff preserved 39 pane shell PIDs on magno-1 and 55 on trivox.
