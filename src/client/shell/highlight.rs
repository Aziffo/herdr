use super::*;
use ratatui::style::Color;

/// Highlight choices reported by aziffo.highlight as pane/workspace metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HighlightColor {
    Red,
    Peach,
    Yellow,
    Green,
    Teal,
    Blue,
    Mauve,
    Gray,
}

impl HighlightColor {
    pub const ALL: [Self; 8] = [
        Self::Red,
        Self::Peach,
        Self::Yellow,
        Self::Green,
        Self::Teal,
        Self::Blue,
        Self::Mauve,
        Self::Gray,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Peach => "peach",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::Teal => "teal",
            Self::Blue => "blue",
            Self::Mauve => "mauve",
            Self::Gray => "gray",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Red => "Red",
            Self::Peach => "Peach",
            Self::Yellow => "Yellow",
            Self::Green => "Green",
            Self::Teal => "Teal",
            Self::Blue => "Blue",
            Self::Mauve => "Mauve",
            Self::Gray => "Gray",
        }
    }
}

impl Palette {
    pub(in crate::client::shell) fn highlight_color(&self, color: HighlightColor) -> Color {
        match color {
            HighlightColor::Red => self.red,
            HighlightColor::Peach => self.peach,
            HighlightColor::Yellow => self.yellow,
            HighlightColor::Green => self.green,
            HighlightColor::Teal => self.teal,
            HighlightColor::Blue => self.blue,
            HighlightColor::Mauve => self.mauve,
            HighlightColor::Gray => self.overlay0,
        }
    }
}

pub(super) fn token_color(tokens: &[(String, String)], key: &str) -> Option<HighlightColor> {
    let value = tokens.iter().find(|(name, _)| name == key)?.1.as_str();
    HighlightColor::ALL
        .into_iter()
        .find(|color| color.label() == value)
}

impl ClientShellState {
    pub(super) fn apply_context_highlight(
        &mut self,
        target: ClientContextMenuTarget,
        color: Option<HighlightColor>,
        outcome: &mut ClientShellInput,
    ) {
        use crate::api::schema::{Method, PluginActionInvokeParams, PluginInvocationContext};
        let (kind, workspace_id, tab_id, focused_pane_id) = match target {
            ClientContextMenuTarget::Workspace { workspace_id, .. } => {
                ("workspace", workspace_id, None, None)
            }
            ClientContextMenuTarget::Tab {
                workspace_id,
                tab_id,
            } => ("tab", workspace_id, Some(tab_id), None),
            ClientContextMenuTarget::Pane {
                workspace_id,
                pane_id,
                ..
            } => {
                let tab_id = self
                    .snapshot
                    .as_deref()
                    .and_then(|snapshot| snapshot.panes.iter().find(|pane| pane.pane_id == pane_id))
                    .map(|pane| pane.tab_id.clone());
                ("pane", workspace_id, tab_id, Some(pane_id))
            }
        };
        self.push_endpoint_method(
            Method::PluginActionInvoke(PluginActionInvokeParams {
                plugin_id: Some("aziffo.highlight".into()),
                action_id: format!(
                    "{kind}-{}",
                    color.map(HighlightColor::key).unwrap_or("clear")
                ),
                context: Some(PluginInvocationContext {
                    workspace_id: Some(workspace_id),
                    workspace_label: None,
                    workspace_cwd: None,
                    worktree: None,
                    tab_id,
                    tab_label: None,
                    focused_pane_id,
                    focused_pane_cwd: None,
                    focused_pane_agent: None,
                    focused_pane_status: None,
                    selected_text: None,
                    invocation_source: Some("context_menu".into()),
                    correlation_id: None,
                    clicked_url: None,
                    link_handler_id: None,
                }),
            }),
            outcome,
        );
    }
}
