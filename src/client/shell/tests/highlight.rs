use super::*;

fn agents_snapshot() -> ClientShellSnapshot {
    let mut value = snapshot();
    let mut sibling = value.panes[0].clone();
    sibling.pane_id = "pane_2".into();
    sibling.focused = false;
    value.panes.push(sibling);
    value.agents = ["pane_1", "pane_2"]
        .into_iter()
        .map(|id| ClientShellAgent {
            pane_id: id.into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("simpleLoginlocal".into()),
            display_agent: None,
            agent: None,
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: AgentStatus::Idle,
            state_change_seq: 1,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: id == "pane_1",
        })
        .collect();
    value
}

fn menu_state() -> ClientShellState {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(agents_snapshot()));
    state.set_pane_surface(surface());
    state.compose(106, 30).unwrap();
    let rect = state
        .hits
        .agents
        .iter()
        .find(|(_, id)| id == "pane_2")
        .unwrap()
        .0;
    let result = state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Right),
        column: rect.x + 1,
        row: rect.y,
        modifiers: KeyModifiers::empty(),
    })]);
    assert!(
        result.actions.is_empty(),
        "opening a menu must not change focus"
    );
    let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_mut() else {
        panic!("agent context menu")
    };
    assert!(
        matches!(&menu.target, ClientContextMenuTarget::Pane {pane_id, ..} if pane_id == "pane_2")
    );
    menu.highlighted = menu
        .items()
        .iter()
        .position(|item| item.has_submenu())
        .unwrap();
    state
}

fn assert_choice(result: &ClientShellInput, expected: &str) {
    let [ClientShellAction::Endpoint { request, .. }] = &result.actions[..] else {
        panic!("one targeted action")
    };
    let crate::api::schema::Method::PaneHighlight(params) = &request.method else {
        panic!("native highlight action")
    };
    assert_eq!(params.pane_id, "pane_2");
    let color = expected.strip_prefix("pane-").unwrap();
    assert_eq!(params.color.as_deref(), (color != "clear").then_some(color));
}

#[test]
fn highlight_duplicate_agent_keyboard_selection_and_clear_target_only_clicked_pane() {
    let mut state = menu_state();
    state.handle_input_bytes(b"\x1b[C");
    state.handle_input_bytes(b"\x1b[B");
    assert_choice(&state.handle_input_bytes(b"\r"), "pane-red");
    let mut state = menu_state();
    state.handle_input_bytes(b"\x1b[C");
    assert_choice(&state.handle_input_bytes(b"\r"), "pane-clear");
}

#[test]
fn highlight_duplicate_agent_mouse_selection_targets_only_clicked_pane() {
    let mut state = menu_state();
    state.compose(106, 30).unwrap();
    let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_ref() else {
        panic!()
    };
    let row = state
        .hits
        .context_menu_rows
        .iter()
        .find(|(_, i)| *i == menu.highlighted)
        .unwrap()
        .0;
    state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: row.x,
        row: row.y,
        modifiers: KeyModifiers::empty(),
    })]);
    state.compose(106, 30).unwrap();
    let swatch = state
        .hits
        .context_submenu_rows
        .iter()
        .find(|(_, i)| *i == 6)
        .unwrap()
        .0;
    let result = state.handle_raw_events(vec![RawInputEvent::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: swatch.x,
        row: swatch.y,
        modifiers: KeyModifiers::empty(),
    })]);
    assert_choice(&result, "pane-blue");
}

#[test]
fn highlight_escape_cancels_and_tiny_menus_remain_in_bounds() {
    let mut state = menu_state();
    state.handle_input_bytes(b"\x1b[C");
    for (w, h) in [(1, 1), (8, 4), (17, 9), (30, 12)] {
        let Some(ClientShellOverlay::ContextMenu(menu)) = state.overlay.as_ref() else {
            panic!()
        };
        let mut buffer = Buffer::empty(Rect::new(0, 0, w, h));
        if let Some(rendered) =
            super::super::render::render_context_menu(&mut buffer, menu, &state.config.palette)
        {
            for (rect, _) in rendered
                .menu_rows
                .iter()
                .chain(rendered.submenu_rows.iter())
            {
                assert!(rect.right() <= w && rect.bottom() <= h);
            }
        }
    }
    assert!(state.handle_input_bytes(b"\x1b").actions.is_empty());
    assert!(state.handle_input_bytes(b"\x1b").actions.is_empty());
    assert!(state.overlay.is_none());
}

#[test]
fn highlight_agent_rendering_does_not_inherit_tab_or_workspace_color() {
    let mut snapshot = agents_snapshot();
    snapshot.agents[0].tokens = vec![
        ("highlight_tab".into(), "Red".into()),
        ("highlight_workspace".into(), "Red".into()),
    ];
    snapshot.agents[1].tokens = vec![("highlight_pane".into(), "Blue".into())];
    let config = ClientShellConfig::from_config(&Config::default());
    for (id, expected) in [("pane_1", None), ("pane_2", Some(HighlightColor::Blue))] {
        let row = super::super::agent_sidebar::agent_row(&snapshot, id, &config, None).unwrap();
        assert_eq!(row.highlight, expected);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 30, 3));
        super::super::agent_sidebar::render_agent_row(
            &mut buffer,
            Rect::new(0, 0, 30, 3),
            &row,
            &config,
        );
        assert_eq!(buffer[(0, 0)].symbol() == "▍", expected.is_some());
    }
    let area = Rect::new(0, 0, 4, 30);
    let mut buffer = Buffer::empty(area);
    let mut hits = ShellHitMap::default();
    super::super::sidebar::render_collapsed_sidebar(
        &mut buffer,
        area,
        &snapshot,
        &config,
        None,
        &mut hits,
    );
    let row = hits.agents.iter().find(|(_, id)| id == "pane_2").unwrap().0;
    assert_eq!(buffer[(row.x, row.y)].fg, config.palette.blue);
}

#[test]
fn highlight_does_not_target_inactive_machine_with_colliding_pane_id() {
    let mut state = menu_state();
    let row = state.hits.agents[0].0;
    state.hits.endpoint_agents = vec![(
        row,
        ClientEndpointId::Ssh(
            crate::client::endpoint::ProfileId::parse("0123456789abcdef0123456789abcdef").unwrap(),
        ),
        "pane_1".into(),
    )];
    assert_eq!(state.active_endpoint_agent_pane_at((row.x, row.y)), None);
}

#[test]
#[ignore = "manual fixed-geometry agent highlight scaling profile"]
fn render_scale_profile_agent_highlight() {
    for count in [1, 15] {
        let config = ClientShellConfig::from_config(&Config::default());
        let mut snapshot = agents_snapshot();
        let agent = snapshot.agents[0].clone();
        snapshot.agents = (0..count)
            .map(|index| {
                let mut agent = agent.clone();
                agent.pane_id = format!("pane_{index}");
                agent.tokens = vec![("unrelated".into(), "Blue".into())];
                agent
            })
            .collect();
        let area = Rect::new(0, 0, 40, 50);
        let mut buffer = Buffer::empty(area);
        let mut scroll = 0;
        let mut hits = ShellHitMap::default();
        let mut durations = Vec::new();
        for highlighted in [false, true] {
            if highlighted {
                for agent in &mut snapshot.agents {
                    agent.tokens[0].0 = "highlight_pane".into();
                }
            }
            let start = std::time::Instant::now();
            for _ in 0..500 {
                hits.agents.clear();
                super::super::agent_sidebar::render_agent_panel(
                    &mut buffer,
                    area,
                    &snapshot,
                    &config,
                    &mut scroll,
                    &mut hits,
                );
                std::hint::black_box(&buffer);
            }
            durations.push(start.elapsed());
        }
        eprintln!("highlight profile: agents={count} geometry=40x50 iterations=500 plain={:?} highlighted={:?}", durations[0], durations[1]);
    }
}
