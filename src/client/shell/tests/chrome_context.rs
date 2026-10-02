use super::*;

#[test]
fn tab_overflow_controls_scroll_the_client_owned_tab_bar() {
    let mut snapshot = snapshot();
    snapshot.tabs.extend((2..=8).map(|number| ClientShellTab {
        highlight: None,
        tab_id: format!("tab_{number}"),
        workspace_id: "ws_1".into(),
        number,
        label: number.to_string(),
        custom_label: false,
        zoomed: false,
        focused: false,
        agent_status: AgentStatus::Idle,
    }));
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot));
    state.set_pane_surface(surface());
    state.compose(80, 20).expect("overflow tab bar");

    assert!(state.hits.tab_scroll_right.width > 0);
    let scroll_right = state.hits.tab_scroll_right;
    let outcome =
        state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: scroll_right.x + 1,
            row: scroll_right.y,
            modifiers: KeyModifiers::empty(),
        })]);
    assert!(outcome.repaint);
    assert_eq!(state.tab_scroll, 1);

    let mut update = state.snapshot.as_deref().expect("snapshot").clone();
    update.focused_tab_id = Some("tab_8".into());
    for tab in &mut update.tabs {
        tab.focused = tab.tab_id == "tab_8";
    }
    state.set_snapshot(Box::new(update));
    state.compose(80, 20).expect("focused overflow tab");
    assert!(state.hits.tabs.iter().any(|(_, tab_id)| tab_id == "tab_8"));

    state.compose(300, 20).expect("tabs without overflow");
    assert_eq!(state.tab_scroll, 0);
    assert_eq!(state.hits.tabs.len(), 8);
    state.compose(80, 20).expect("focused tab after narrowing");
    assert!(state.hits.tabs.iter().any(|(_, tab_id)| tab_id == "tab_8"));
}

#[test]
fn focused_last_overflow_tab_shows_its_full_label() {
    let mut projected = snapshot();
    let labels = [
        "1",
        "Laiza Portfolio Site",
        "linkedin posts",
        "update cv",
        "nvim",
        "brother",
        "day organiser",
        "nvim test",
    ];
    projected.tabs = labels
        .iter()
        .enumerate()
        .map(|(index, label)| ClientShellTab {
            highlight: None,
            tab_id: format!("tab_{}", index + 1),
            workspace_id: "ws_1".into(),
            number: index + 1,
            label: (*label).into(),
            custom_label: index > 0,
            zoomed: false,
            focused: index == 7,
            agent_status: AgentStatus::Idle,
        })
        .collect();
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    for number in [8, 7, 8] {
        let tab_id = format!("tab_{number}");
        projected.focused_tab_id = Some(tab_id.clone());
        projected.workspaces[0].active_tab_id = tab_id.clone();
        projected.panes[0].tab_id = tab_id.clone();
        for tab in &mut projected.tabs {
            tab.focused = tab.tab_id == tab_id;
        }
        state.set_snapshot(Box::new(projected.clone()));
        state.set_pane_surface(surface());
        let frame = state
            .compose(133, 20)
            .expect("reporter's overflowing strip");
        assert_eq!(
            state.hits.new_tab.right() - state.hits.tab_scroll_left.x,
            107
        );
        let rect = state
            .hits
            .tabs
            .iter()
            .find(|(_, id)| id == &tab_id)
            .expect("focused tab")
            .0;
        let text = (rect.x..rect.right())
            .map(|x| {
                frame.cells[(rect.y * frame.width + x) as usize]
                    .symbol
                    .as_str()
            })
            .collect::<String>();
        assert!(
            text.contains(labels[number - 1]),
            "focused tab rendered as {text:?}, rect={rect:?}"
        );
    }

    // Manual scrolling must be able to reveal the rest of a partially drawn last tab.
    let end_scroll = state.tab_scroll;
    let left = state.hits.tab_scroll_left;
    state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: left.x + 1,
        row: left.y,
        modifiers: KeyModifiers::empty(),
    })]);
    let frame = state.compose(133, 20).expect("manual scroll left");
    assert_eq!(state.tab_scroll, end_scroll - 1);
    let right = state.hits.tab_scroll_right;
    assert_eq!(
        frame.cells[(right.y * frame.width + right.x + 1) as usize].fg,
        crate::protocol::color_to_u32(state.config.palette.overlay1),
        "right arrow stays enabled while the final tab is clipped"
    );
    state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: right.x + 1,
        row: right.y,
        modifiers: KeyModifiers::empty(),
    })]);
    let frame = state.compose(133, 20).expect("manual scroll right");
    assert_eq!(state.tab_scroll, end_scroll);
    assert!(frame_rows(&frame)[0].contains("nvim test"));
    assert_eq!(
        frame.cells[(right.y * frame.width + right.x + 1) as usize].fg,
        crate::protocol::color_to_u32(state.config.palette.overlay0),
        "right arrow dims at the useful scroll limit"
    );
}

#[test]
fn focused_workspace_change_reveals_new_workspace_in_full_sidebar() {
    let mut initial = snapshot();
    let template = initial.workspaces[0].clone();
    initial.workspaces = (1..=12)
        .map(|number| ClientShellWorkspace {
            highlight: None,
            workspace_id: format!("ws_{number}"),
            number,
            label: format!("space-{number}"),
            branch: None,
            focused: number == 1,
            ..template.clone()
        })
        .collect();

    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(initial));
    state.set_pane_surface(surface());
    state.compose(106, 20).expect("full sidebar");
    assert!(state.hits.workspace_max_scroll > 0);
    assert!(state
        .hits
        .workspaces
        .iter()
        .all(|hit| hit.workspace_id != "ws_12"));

    let mut update = state.snapshot.as_deref().expect("snapshot").clone();
    update.revision = 2;
    update.focused_workspace_id = Some("ws_12".into());
    for workspace in &mut update.workspaces {
        workspace.focused = workspace.workspace_id == "ws_12";
    }
    let mut updated_surface = surface();
    updated_surface.projection_revision = 2;
    state.set_snapshot(Box::new(update));
    state.set_pane_surface(updated_surface);
    state.compose(106, 2).expect("zero-height workspace body");
    assert!(state.reveal_focused_workspace);
    state.compose(106, 20).expect("updated full sidebar");

    assert!(state
        .hits
        .workspaces
        .iter()
        .any(|hit| hit.workspace_id == "ws_12"));
}

#[test]
fn client_owned_sidebar_dividers_resize_live() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface());
    state.compose(106, 30).expect("expanded sidebar");
    assert!(state.hits.machines.is_empty());
    let workspace_body = state.hits.workspace_body;
    let needless_scroll =
        state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: workspace_body.x,
            row: workspace_body.y,
            modifiers: KeyModifiers::empty(),
        })]);
    assert_eq!(state.hits.workspace_max_scroll, 0);
    assert_eq!(state.workspace_scroll, 0);
    assert!(!needless_scroll.repaint);
    let width_divider = state.hits.sidebar_divider;
    state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: width_divider.x,
        row: width_divider.y + 2,
        modifiers: KeyModifiers::empty(),
    })]);
    let resize =
        state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 31,
            row: width_divider.y + 2,
            modifiers: KeyModifiers::empty(),
        })]);
    assert_eq!(state.sidebar_width, 32);
    assert!(state.sidebar_width_manual);
    assert!(resize.repaint);
    assert!(resize.resize);
    let waiting_frame = state.compose(106, 30).expect("waiting for resized surface");
    let waiting_text: String = waiting_frame
        .cells
        .iter()
        .map(|cell| cell.symbol.as_str())
        .collect();
    assert!(
        waiting_text.contains(" spaces"),
        "local sidebar must keep spaces while resizing: {waiting_text}"
    );
    assert!(!waiting_text.contains(" machines"));
    assert!(!waiting_text.contains("Select a connected machine"));
    assert!(!waiting_text.contains("LIVE"));
    assert!(waiting_frame.cursor.is_none());
    assert!(state.pane_surface.is_none());
    assert!(state.hits.panes.is_empty());
    assert!(state.hits.pane_splits.is_empty());
    assert!(state.hits.machines.is_empty());
    assert_eq!(state.hits.sidebar_divider.x, 31);
    assert_eq!(state.hits.workspaces[0].workspace_id, "ws_1");

    let next_resize =
        state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: 32,
            row: width_divider.y + 2,
            modifiers: KeyModifiers::empty(),
        })]);
    assert!(next_resize.resize);
    state.compose(106, 30).expect("continued resize");
    assert_eq!(state.hits.sidebar_divider.x, 32);
    state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: 32,
        row: width_divider.y + 2,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(state.chrome_drag.is_none());

    state.set_pane_surface(surface());
    let recovered_frame = state.compose(106, 30).expect("resized sidebar");
    let recovered_text: String = recovered_frame
        .cells
        .iter()
        .map(|cell| cell.symbol.as_str())
        .collect();
    assert!(recovered_text.contains(" spaces"));
    assert!(recovered_text.contains("LIVE"));
    assert!(!state.hits.panes.is_empty());
    let section_divider = state.hits.sidebar_section_divider;
    state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: section_divider.x + 2,
        row: section_divider.y,
        modifiers: KeyModifiers::empty(),
    })]);
    let split = state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: section_divider.x + 2,
        row: 20,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(state.sidebar_section_split > 0.6);
    assert!(split.repaint);
    assert!(!split.resize);
}

#[test]
fn context_menus_capture_stable_targets_and_route_actions() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface());
    state.compose(106, 20).expect("composed frame");

    let workspace = state.hits.workspaces[0].rect;
    let open_workspace_menu =
        state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Right),
            column: workspace.x + 2,
            row: workspace.y,
            modifiers: KeyModifiers::empty(),
        })]);
    assert!(open_workspace_menu.actions.is_empty());
    assert!(matches!(
        state.overlay,
        Some(ClientShellOverlay::ContextMenu(ClientContextMenuOverlay {
            target: ClientContextMenuTarget::Workspace { ref workspace_id, .. },
            ..
        })) if workspace_id == "ws_1"
    ));
    let workspace_items = match state.overlay.as_ref() {
        Some(ClientShellOverlay::ContextMenu(menu)) => menu.items(),
        _ => panic!("workspace context menu"),
    };
    assert!(workspace_items
        .iter()
        .any(|item| item.action == ClientContextMenuAction::NewWorktree));
    state.compose(106, 20).expect("workspace context menu");
    let rename = state.hits.context_menu_rows[0].0;
    state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: rename.x + 1,
        row: rename.y,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(matches!(
        state.overlay,
        Some(ClientShellOverlay::Rename(ClientRenameOverlay {
            target: ClientRenameTarget::Workspace { ref workspace_id },
            ..
        })) if workspace_id == "ws_1"
    ));

    state.overlay = None;
    state.compose(106, 20).expect("composed frame");
    let pane = state.hits.panes[0].rect;
    state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: pane.x + 1,
        row: pane.y,
        modifiers: KeyModifiers::empty(),
    })]);
    state.compose(106, 20).expect("pane context menu");
    let split_index = match state.overlay.as_ref() {
        Some(ClientShellOverlay::ContextMenu(menu)) => menu
            .items()
            .iter()
            .position(|item| item.action == ClientContextMenuAction::SplitRight)
            .expect("split right item"),
        _ => panic!("pane context menu"),
    };
    let split = state.hits.context_menu_rows[split_index].0;
    let outcome =
        state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: split.x + 1,
            row: split.y,
            modifiers: KeyModifiers::empty(),
        })]);
    let [ClientShellAction::Endpoint { request, .. }] = &outcome.actions[..] else {
        panic!("pane split context action should use endpoint API");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::PaneSplit(params)
            if params.target_pane_id.as_deref() == Some("pane_1")
                && params.direction == crate::api::schema::SplitDirection::Right
    ));
}

#[test]
fn global_menu_opens_from_sidebar_and_routes_client_actions() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface());
    state.compose(106, 30).expect("shell frame");
    let launcher = state.hits.global_launcher;
    assert_ne!(launcher, Rect::default());

    let open = state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: launcher.x,
        row: launcher.y,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(open.repaint);
    let menu = state.compose(106, 30).expect("global menu");
    let text = menu
        .cells
        .chunks(menu.width as usize)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("settings"));
    assert!(text.contains("keybinds"));
    assert!(text.contains("reload config"));
    assert!(text.contains("detach"));

    let keybinds = state.hits.global_menu_rows[1].0;
    let help = state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: keybinds.x,
        row: keybinds.y,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(help.actions.is_empty());
    assert!(matches!(state.overlay, Some(ClientShellOverlay::Help(_))));

    state.overlay = Some(ClientShellOverlay::GlobalMenu(ClientGlobalMenuOverlay {
        highlighted: 3,
    }));
    let detach = state.handle_input_bytes(b"\r");
    assert!(detach.detach);
    assert!(state.overlay.is_none());
}

#[test]
fn new_tab_overlay_owns_text_cursor_and_submits_public_api_request() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface());
    let mut open = ClientShellInput::default();
    state.record_binding(
        crate::input::KeybindMatch::Action(crate::input::KeybindAction::NewTab),
        &mut open,
    );
    assert!(open.actions.is_empty());
    let frame = state.compose(106, 20).expect("new tab overlay");
    let text = frame
        .cells
        .chunks(frame.width as usize)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("new tab"));
    assert!(text.contains("save"));
    let restored = frame.to_ratatui_buffer().expect("overlay frame");
    assert!(!restored
        .cell((26, 7))
        .expect("overlay title cell")
        .modifier
        .contains(Modifier::DIM));
    assert!(frame.cursor.as_ref().is_some_and(|cursor| cursor.visible));

    assert!(state.handle_input_bytes(b"logs").actions.is_empty());
    let create = state.handle_input_bytes(b"\r");
    let [ClientShellAction::Endpoint { request, .. }] = &create.actions[..] else {
        panic!("new tab save should use endpoint API");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::TabCreate(params)
            if params.workspace_id.as_deref() == Some("ws_1")
                && params.label.as_deref() == Some("logs")
    ));
    assert!(state.overlay.is_none());
}

#[test]
fn close_confirmation_error_becomes_client_owned_overlay_and_stable_group_close() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface());
    let mut close = ClientShellInput::default();
    state.record_binding(
        crate::input::KeybindMatch::Action(crate::input::KeybindAction::ClosePane),
        &mut close,
    );
    let [ClientShellAction::Endpoint { request, .. }] = &close.actions[..] else {
        panic!("pane close should use endpoint API");
    };
    let request_id = request.id.clone();
    assert!(
        state
            .handle_endpoint_result(
                "boot-1",
                &request_id,
                Err(ClientShellEndpointError {
                    code: Some("confirmation_required".into()),
                    message: "confirmation required".into(),
                }),
            )
            .0
    );
    let frame = state.compose(106, 20).expect("confirmation overlay");
    let text = frame
        .cells
        .chunks(frame.width as usize)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("Close workspace?"));
    assert!(text.contains("1 pane"));

    let confirm = state.handle_input_bytes(b"\r");
    let [ClientShellAction::Endpoint { request, .. }] = &confirm.actions[..] else {
        panic!("confirmation should use endpoint API");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::WorkspaceClose(params)
            if params.workspace_id == "ws_1" && params.close_group
    ));
}

#[test]
fn highlight_context_menu_offers_submenu_for_tabs_and_workspaces_only() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot()));
    for tab in [true, false] {
        if tab {
            state.open_tab_context_menu("tab_1".into(), 0, 0);
        } else {
            state.open_workspace_context_menu("ws_1".into(), 0, 0);
        }
        let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_ref() else {
            panic!("menu");
        };
        assert!(menu.items().iter().any(|item| item.label == "Highlight ▸"));
    }
    state.open_pane_context_menu("pane_1".into(), 0, 0);
    let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_ref() else {
        panic!("pane menu");
    };
    assert!(menu.items().iter().all(|item| item.label != "Highlight ▸"));
}

fn highlight_menu_state(
    tab: bool,
    color: Option<crate::workspace::HighlightColor>,
) -> ClientShellState {
    let mut snapshot = snapshot();
    snapshot.tabs[0].highlight = color;
    snapshot.workspaces[0].highlight = color;
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot));
    state.set_pane_surface(surface());
    state.compose(106, 20).unwrap();
    let rect = if tab {
        state.hits.tabs[0].0
    } else {
        state.hits.workspaces[0].rect
    };
    state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: rect.x + 1,
        row: rect.y,
        modifiers: KeyModifiers::empty(),
    })]);
    let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_mut() else {
        panic!("menu");
    };
    menu.highlighted = menu
        .items()
        .iter()
        .position(|item| item.has_submenu())
        .unwrap();
    state
}

#[test]
fn highlight_keyboard_opens_returns_navigates_and_applies_without_changing_focus() {
    use crate::workspace::HighlightColor;
    for tab in [true, false] {
        for open in [b"\x1b[C".as_slice(), b"\r".as_slice()] {
            let mut state = highlight_menu_state(tab, Some(HighlightColor::Blue));
            state.handle_input_bytes(open);
            assert!(matches!(
                state.overlay,
                Some(ClientShellOverlay::ContextMenu(ClientContextMenuOverlay {
                    open_submenu: true,
                    submenu_highlighted: 6,
                    ..
                }))
            ));
            state.handle_input_bytes(b"\x1b[D");
            assert!(matches!(
                state.overlay,
                Some(ClientShellOverlay::ContextMenu(ClientContextMenuOverlay {
                    open_submenu: false,
                    ..
                }))
            ));
            state.handle_input_bytes(b"\x1b[C");
            state.handle_raw_events(vec![RawInputEvent::Key(crate::input::TerminalKey::new(
                KeyCode::Esc,
                KeyModifiers::empty(),
            ))]);
            assert!(matches!(
                state.overlay,
                Some(ClientShellOverlay::ContextMenu(ClientContextMenuOverlay {
                    open_submenu: false,
                    ..
                }))
            ));
            state.handle_input_bytes(b"\r");
            state.handle_input_bytes(b"\x1b[B");
            state.handle_input_bytes(b"\x1b[A");
            state.handle_input_bytes(b"\x1b[B");
            let result = state.handle_input_bytes(b"\r");
            assert!(state.overlay.is_none());
            let [ClientShellAction::Endpoint { request, .. }] = &result.actions[..] else {
                panic!("one highlight request");
            };
            if tab {
                assert!(
                    matches!(&request.method, crate::api::schema::Method::TabHighlight(params)
                    if params.tab_id == "tab_1" && params.highlight.as_deref() == Some("mauve"))
                );
            } else {
                assert!(
                    matches!(&request.method, crate::api::schema::Method::WorkspaceHighlight(params)
                    if params.workspace_id == "ws_1" && params.highlight.as_deref() == Some("mauve"))
                );
            }
        }
    }
}

#[test]
fn highlight_hover_swatches_check_and_none_click_close_the_whole_menu() {
    use crate::workspace::HighlightColor;
    for tab in [true, false] {
        let mut state = highlight_menu_state(tab, Some(HighlightColor::Red));
        state.compose(106, 20).unwrap();
        let row = *state.hits.context_menu_rows.last().unwrap();
        state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: row.0.x,
            row: row.0.y,
            modifiers: KeyModifiers::empty(),
        })]);
        let frame = state.compose(106, 20).unwrap();
        assert_eq!(state.hits.context_submenu_rows.len(), 9);
        let red = state.hits.context_submenu_rows[1].0;
        assert_eq!(
            frame.cells[(red.y * frame.width + red.x) as usize].bg,
            crate::protocol::color_to_u32(state.config.palette.red)
        );
        assert_eq!(
            frame.cells[(red.y * frame.width + red.right() - 2) as usize].symbol,
            "✓"
        );
        assert!(frame.cursor.is_none());
        let none = state.hits.context_submenu_rows[0].0;
        let result = state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: none.x,
            row: none.y,
            modifiers: KeyModifiers::empty(),
        })]);
        assert!(state.overlay.is_none());
        let [ClientShellAction::Endpoint { request, .. }] = &result.actions[..] else {
            panic!("highlight request");
        };
        assert!(
            matches!(&request.method, crate::api::schema::Method::TabHighlight(params) if tab && params.highlight.is_none())
                || matches!(&request.method, crate::api::schema::Method::WorkspaceHighlight(params) if !tab && params.highlight.is_none())
        );
    }
}

#[test]
fn highlight_submenu_clamps_left_and_scrolls_in_short_terminals() {
    let mut state = highlight_menu_state(true, None);
    if let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_mut() {
        menu.x = 105;
        menu.y = 19;
    }
    state.handle_input_bytes(b"\x1b[C");
    state.compose(106, 20).unwrap();
    assert!(state.hits.context_submenu_area.right() <= state.hits.context_menu_area.x);
    assert!(state.hits.context_submenu_area.bottom() <= 20);
    for _ in 0..8 {
        state.handle_input_bytes(b"\x1b[B");
    }
    state.compose(40, 6).unwrap();
    assert!(state.hits.context_submenu_area.right() <= 40);
    assert!(state.hits.context_submenu_area.bottom() <= 6);
    assert!(state
        .hits
        .context_submenu_rows
        .iter()
        .any(|(_, index)| *index == 8));
    let result = state.handle_input_bytes(b"\r");
    assert!(
        matches!(&result.actions[0], ClientShellAction::Endpoint { request, .. }
        if matches!(&request.method, crate::api::schema::Method::TabHighlight(params) if params.highlight.as_deref() == Some("gray")))
    );
}

#[test]
fn highlight_outside_click_closes_and_hovering_other_parent_rows_returns() {
    let mut state = highlight_menu_state(true, None);
    state.handle_input_bytes(b"\x1b[C");
    state.compose(106, 20).unwrap();
    let row = state.hits.context_menu_rows[0].0;
    state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: row.x,
        row: row.y,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(matches!(
        state.overlay,
        Some(ClientShellOverlay::ContextMenu(ClientContextMenuOverlay {
            open_submenu: false,
            highlighted: 0,
            ..
        }))
    ));
    state.move_context_menu_selection(3);
    state.handle_input_bytes(b"\x1b[C");
    state.compose(106, 20).unwrap();
    state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 105,
        row: 19,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(state.overlay.is_none());
}

#[test]
fn highlight_tabs_and_sidebar_follow_the_theme_and_clear_to_normal() {
    use crate::workspace::HighlightColor;
    let mut snapshot = snapshot();
    snapshot.tabs[0].highlight = Some(HighlightColor::Peach);
    snapshot.workspaces[0].highlight = Some(HighlightColor::Teal);
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot.clone()));
    state.set_pane_surface(surface());
    let frame = state.compose(106, 20).unwrap();
    let tab = state.hits.tabs[0].0;
    assert_eq!(
        frame.cells[(tab.y * frame.width + tab.x) as usize].bg,
        crate::protocol::color_to_u32(state.config.palette.peach)
    );
    let workspace = state.hits.workspaces[0].rect;
    assert!((workspace.x..workspace.right()).any(|x| frame.cells
        [(workspace.y * frame.width + x) as usize]
        .symbol
        == "▍"
        && frame.cells[(workspace.y * frame.width + x) as usize].fg
            == crate::protocol::color_to_u32(state.config.palette.teal)));
    snapshot.tabs[0].highlight = None;
    snapshot.workspaces[0].highlight = None;
    state.set_snapshot(Box::new(snapshot));
    let cleared = state.compose(106, 20).unwrap();
    let tab = state.hits.tabs[0].0;
    assert_eq!(
        cleared.cells[(tab.y * cleared.width + tab.x) as usize].bg,
        crate::protocol::color_to_u32(state.config.palette.accent)
    );
}

fn highlight_agent_snapshot() -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    snapshot.agents = vec![ClientShellAgent {
        pane_id: "pane_1".into(),
        workspace_id: "ws_1".into(),
        tab_id: "tab_1".into(),
        name: Some("agent".into()),
        display_agent: None,
        agent: None,
        title: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: AgentStatus::Idle,
        state_change_seq: 1,
        state_labels: Vec::new(),
        tokens: Vec::new(),
        focused: true,
    }];
    snapshot
}

#[test]
fn highlight_agent_rows_prefer_tab_color_then_workspace_and_clear() {
    use crate::workspace::HighlightColor;
    let mut snapshot = highlight_agent_snapshot();
    let config = ClientShellConfig::from_config(&Config::default());
    for (tab_color, workspace_color, expected) in [
        (
            Some(HighlightColor::Blue),
            Some(HighlightColor::Teal),
            Some(HighlightColor::Blue),
        ),
        (None, Some(HighlightColor::Teal), Some(HighlightColor::Teal)),
        (None, None, None),
    ] {
        snapshot.tabs[0].highlight = tab_color;
        snapshot.workspaces[0].highlight = workspace_color;
        let row =
            super::super::agent_sidebar::agent_row(&snapshot, "pane_1", &config, None).unwrap();
        assert_eq!(row.highlight, expected);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 3));
        super::super::agent_sidebar::render_agent_row(
            &mut buffer,
            Rect::new(0, 0, 30, 3),
            &row,
            &config,
        );
        if let Some(color) = expected {
            assert_eq!(buffer[(0, 0)].symbol(), "▍");
            assert_eq!(buffer[(0, 0)].fg, config.palette.highlight_color(color));
        } else {
            assert_ne!(buffer[(0, 0)].symbol(), "▍");
        }
    }
}

#[test]
fn highlight_closed_submenu_does_not_accept_stale_mouse_hits_in_the_same_batch() {
    let mut state = highlight_menu_state(true, None);
    state.handle_input_bytes(b"\x1b[C");
    state.compose(106, 20).unwrap();
    let swatch = state.hits.context_submenu_rows[1].0;
    let result = state.handle_raw_events(vec![
        RawInputEvent::Key(crate::input::TerminalKey::new(
            KeyCode::Esc,
            KeyModifiers::empty(),
        )),
        RawInputEvent::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: swatch.x,
            row: swatch.y,
            modifiers: KeyModifiers::empty(),
        }),
    ]);
    assert!(result.actions.is_empty());
    assert!(state.overlay.is_none());
}

#[test]
fn highlight_right_click_on_agent_row_opens_its_tab_menu_without_focus() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(highlight_agent_snapshot()));
    state.set_pane_surface(surface());
    state.compose(106, 30).unwrap();
    let row = state.hits.agents[0].0;
    let result = state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: row.x + 1,
        row: row.y,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(result.actions.is_empty());
    let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_mut() else {
        panic!("agent row must open its tab menu");
    };
    assert!(matches!(&menu.target, ClientContextMenuTarget::Tab {tab_id, ..} if tab_id == "tab_1"));
    menu.highlighted = menu
        .items()
        .iter()
        .position(|item| item.has_submenu())
        .unwrap();
    state.handle_input_bytes(b"\x1b[C");
    state.handle_input_bytes(b"\x1b[B");
    let result = state.handle_input_bytes(b"\r");
    let [ClientShellAction::Endpoint { request, .. }] = &result.actions[..] else {
        panic!("one tab highlight request");
    };
    assert!(
        matches!(&request.method, crate::api::schema::Method::TabHighlight(params)
        if params.tab_id == "tab_1" && params.highlight.as_deref() == Some("red"))
    );
}

#[test]
fn highlight_agent_menu_ignores_inactive_endpoints_with_colliding_pane_ids() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(highlight_agent_snapshot()));
    state.set_pane_surface(surface());
    state.compose(106, 30).unwrap();
    let row = state.hits.agents[0].0;
    state.hits.endpoint_agents = vec![(
        row,
        ClientEndpointId::Ssh(
            crate::client::endpoint::ProfileId::parse("0123456789abcdef0123456789abcdef").unwrap(),
        ),
        "pane_1".into(),
    )];
    assert_eq!(state.active_endpoint_agent_tab_at((row.x, row.y)), None);
    state.hits.endpoint_agents[0].1 = state.active_endpoint_id.clone();
    assert_eq!(
        state.active_endpoint_agent_tab_at((row.x, row.y)),
        Some("tab_1".into())
    );
}
