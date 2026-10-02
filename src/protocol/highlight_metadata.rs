//! Optional JSON highlight metadata while preserving frozen binary snapshot fields.
use super::{
    wire::deserialize_client_shell_agent_status, ClientShellTab, ClientShellWorkspace,
    ClientShellWorktree,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ClientShellTabBinary {
    tab_id: String,
    workspace_id: String,
    number: usize,
    label: String,
    custom_label: bool,
    zoomed: bool,
    focused: bool,
    #[serde(deserialize_with = "deserialize_client_shell_agent_status")]
    agent_status: crate::api::schema::AgentStatus,
}

#[derive(Deserialize)]
struct ClientShellTabJson {
    tab_id: String,
    workspace_id: String,
    number: usize,
    label: String,
    custom_label: bool,
    zoomed: bool,
    focused: bool,
    #[serde(deserialize_with = "deserialize_client_shell_agent_status")]
    agent_status: crate::api::schema::AgentStatus,
    #[serde(default)]
    highlight: Option<crate::workspace::HighlightColor>,
}

impl Serialize for ClientShellTab {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let include_highlight = serializer.is_human_readable() && self.highlight.is_some();
        let mut fields =
            serializer.serialize_struct("ClientShellTab", 8 + usize::from(include_highlight))?;
        fields.serialize_field("tab_id", &self.tab_id)?;
        fields.serialize_field("workspace_id", &self.workspace_id)?;
        fields.serialize_field("number", &self.number)?;
        fields.serialize_field("label", &self.label)?;
        fields.serialize_field("custom_label", &self.custom_label)?;
        fields.serialize_field("zoomed", &self.zoomed)?;
        fields.serialize_field("focused", &self.focused)?;
        fields.serialize_field("agent_status", &self.agent_status)?;
        if include_highlight {
            fields.serialize_field("highlight", &self.highlight)?;
        }
        fields.end()
    }
}

impl<'de> Deserialize<'de> for ClientShellTab {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let value = ClientShellTabJson::deserialize(deserializer)?;
            Ok(Self {
                tab_id: value.tab_id,
                workspace_id: value.workspace_id,
                number: value.number,
                label: value.label,
                custom_label: value.custom_label,
                zoomed: value.zoomed,
                focused: value.focused,
                agent_status: value.agent_status,
                highlight: value.highlight,
            })
        } else {
            let value = ClientShellTabBinary::deserialize(deserializer)?;
            Ok(Self {
                tab_id: value.tab_id,
                workspace_id: value.workspace_id,
                number: value.number,
                label: value.label,
                custom_label: value.custom_label,
                zoomed: value.zoomed,
                focused: value.focused,
                agent_status: value.agent_status,
                highlight: None,
            })
        }
    }
}

#[derive(Deserialize)]
struct ClientShellWorkspaceBinary {
    workspace_id: String,
    active_tab_id: String,
    new_workspace_cwd: String,
    number: usize,
    label: String,
    custom_label: bool,
    branch: Option<String>,
    git_ahead_behind: Option<(usize, usize)>,
    tokens: Vec<(String, String)>,
    worktree: Option<ClientShellWorktree>,
    focused: bool,
    #[serde(deserialize_with = "deserialize_client_shell_agent_status")]
    agent_status: crate::api::schema::AgentStatus,
}

#[derive(Deserialize)]
struct ClientShellWorkspaceJson {
    workspace_id: String,
    active_tab_id: String,
    new_workspace_cwd: String,
    number: usize,
    label: String,
    custom_label: bool,
    branch: Option<String>,
    git_ahead_behind: Option<(usize, usize)>,
    tokens: Vec<(String, String)>,
    worktree: Option<ClientShellWorktree>,
    focused: bool,
    #[serde(deserialize_with = "deserialize_client_shell_agent_status")]
    agent_status: crate::api::schema::AgentStatus,
    #[serde(default)]
    highlight: Option<crate::workspace::HighlightColor>,
}

impl Serialize for ClientShellWorkspace {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let include_highlight = serializer.is_human_readable() && self.highlight.is_some();
        let mut fields = serializer
            .serialize_struct("ClientShellWorkspace", 12 + usize::from(include_highlight))?;
        fields.serialize_field("workspace_id", &self.workspace_id)?;
        fields.serialize_field("active_tab_id", &self.active_tab_id)?;
        fields.serialize_field("new_workspace_cwd", &self.new_workspace_cwd)?;
        fields.serialize_field("number", &self.number)?;
        fields.serialize_field("label", &self.label)?;
        fields.serialize_field("custom_label", &self.custom_label)?;
        fields.serialize_field("branch", &self.branch)?;
        fields.serialize_field("git_ahead_behind", &self.git_ahead_behind)?;
        fields.serialize_field("tokens", &self.tokens)?;
        fields.serialize_field("worktree", &self.worktree)?;
        fields.serialize_field("focused", &self.focused)?;
        fields.serialize_field("agent_status", &self.agent_status)?;
        if include_highlight {
            fields.serialize_field("highlight", &self.highlight)?;
        }
        fields.end()
    }
}

impl<'de> Deserialize<'de> for ClientShellWorkspace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let value = ClientShellWorkspaceJson::deserialize(deserializer)?;
            Ok(Self {
                workspace_id: value.workspace_id,
                active_tab_id: value.active_tab_id,
                new_workspace_cwd: value.new_workspace_cwd,
                number: value.number,
                label: value.label,
                custom_label: value.custom_label,
                branch: value.branch,
                git_ahead_behind: value.git_ahead_behind,
                tokens: value.tokens,
                worktree: value.worktree,
                focused: value.focused,
                agent_status: value.agent_status,
                highlight: value.highlight,
            })
        } else {
            let value = ClientShellWorkspaceBinary::deserialize(deserializer)?;
            Ok(Self {
                workspace_id: value.workspace_id,
                active_tab_id: value.active_tab_id,
                new_workspace_cwd: value.new_workspace_cwd,
                number: value.number,
                label: value.label,
                custom_label: value.custom_label,
                branch: value.branch,
                git_ahead_behind: value.git_ahead_behind,
                tokens: value.tokens,
                worktree: value.worktree,
                focused: value.focused,
                agent_status: value.agent_status,
                highlight: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::HighlightColor;

    #[test]
    fn highlight_snapshot_json_is_optional_and_legacy_binary_stays_unchanged() {
        let json = serde_json::json!({"tab_id":"w1:t1", "workspace_id":"w1", "number":1, "label":"shell", "custom_label":false, "zoomed":false, "focused":true, "agent_status":"idle"});
        let mut tab: ClientShellTab = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(tab.highlight, None);
        let baseline = bincode::serde::encode_to_vec(&tab, bincode::config::standard()).unwrap();
        // Literal generation-1 bytes freeze the original field order independently of this serializer.
        assert_eq!(
            baseline,
            vec![5, 119, 49, 58, 116, 49, 2, 119, 49, 1, 5, 115, 104, 101, 108, 108, 0, 0, 1, 0],
            "frozen generation-1 tab bytes"
        );
        tab.highlight = Some(HighlightColor::Mauve);
        let encoded = bincode::serde::encode_to_vec(&tab, bincode::config::standard()).unwrap();
        assert_eq!(encoded, baseline);
        let (legacy, consumed): (ClientShellTab, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(legacy.highlight, None);
        let value = serde_json::to_value(&tab).unwrap();
        assert_eq!(value["highlight"], "mauve");
        assert_eq!(
            serde_json::from_value::<ClientShellTab>(value).unwrap(),
            tab
        );
        let workspace_json = serde_json::json!({"workspace_id":"w1", "active_tab_id":"w1:t1", "new_workspace_cwd":"/tmp", "number":1, "label":"workspace", "custom_label":false, "branch":null, "git_ahead_behind":null, "tokens":[], "worktree":null, "focused":true, "agent_status":"idle"});
        let mut workspace: ClientShellWorkspace = serde_json::from_value(workspace_json).unwrap();
        assert_eq!(workspace.highlight, None);
        let baseline =
            bincode::serde::encode_to_vec(&workspace, bincode::config::standard()).unwrap();
        assert_eq!(
            baseline,
            vec![
                2, 119, 49, 5, 119, 49, 58, 116, 49, 4, 47, 116, 109, 112, 1, 9, 119, 111, 114,
                107, 115, 112, 97, 99, 101, 0, 0, 0, 0, 0, 1, 0
            ],
            "frozen generation-1 workspace bytes"
        );
        workspace.highlight = Some(HighlightColor::Gray);
        assert_eq!(
            bincode::serde::encode_to_vec(&workspace, bincode::config::standard()).unwrap(),
            baseline
        );
        let value = serde_json::to_value(&workspace).unwrap();
        assert_eq!(value["highlight"], "gray");
        assert_eq!(
            serde_json::from_value::<ClientShellWorkspace>(value).unwrap(),
            workspace
        );
    }
}
