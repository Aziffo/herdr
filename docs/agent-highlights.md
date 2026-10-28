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

Pane choices use the additive `pane.highlight` API. They update terminal state
immediately and persist with the session through the existing background writer.
The plugin picker (0.2.1+) uses the same API. Native choices, including an explicit
None, take precedence over legacy plugin tokens, so delayed startup/move hooks
cannot overwrite a newer selection. Colors follow terminals across pane moves,
restart and live handoff. Existing plugin colors remain visible until changed.
Workspace and tab actions still use the plugin's SQLite implementation.

Published binary codecs and existing JSON method shapes are unchanged. An older
server lacking the advertised pane method reports an unavailable action. Custom
sidebar rows should omit `$highlight_pane` to show only the native color marker.

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


## Native pane latency correction (2026-10-28)

The prior plugin action took 13.8–22.2 seconds on the live trivox session, and
concurrent actions could fail with SQLite locking errors. A pane selection now
updates server-owned terminal state directly; the existing background session
writer handles persistence. No plugin process, SQLite transaction or full-session
metadata projection is needed for a native context-menu choice.

Validation of this correction:

- All 4,036 Rust tests pass (15 manual tests skipped), with formatting and clippy
  warnings denied. Maintenance, UI architecture, integration assets and all seven
  documentation contract tests pass. Windows cross-lint still requires the SDK
  unavailable on this Linux deployment host.
- All 22 plugin unit tests pass. Real-server integration passes on both the older
  server fallback and the native fork. The native integration deliberately holds
  a SQLite write lock while changing colors, then checks restart and pane moves.
- Final optimized binary, actual PTY mouse selection: 11.25 ms from click to
  confirmed color. Duplicate agent names remain independent; None clears without
  focus changes. An older plugin projection cannot override either native state.
- Fixed 40x50 rendering, 500 debug iterations: 1 agent 63.42/63.95 ms and 15 agents
  904.34/796.62 ms without/with the marker; observational timings under build load.
- Live magno-1, 12 set/clear requests including a confirming read: median 1.09 ms,
  maximum 5.07 ms. Focus and the sibling pane remain unchanged.
- Removed textual highlight tokens from the deployed custom sidebar rows; only
  the native color indicator remains.

Corrected binary SHA-256:
`5efe4dfae1dfe1af991ca1d6f12d7f53d367a1187945fd0ba30b71f42437128f`.
Plugin 0.2.1 is published on `Aziffo/herdr-highlight` main at `1b12589`.

Deployment verification: magno-1 retained all 39 pane IDs/shell PIDs; trivox
retained all 55. Both existing trivox pane colors were migrated and verified.
Twelve live trivox set/clear requests including confirming reads measured a
1.27 ms median and 51.14 ms maximum. The graphical client was relaunched on the
same corrected binary; both endpoint handshakes succeeded with generation 1.
