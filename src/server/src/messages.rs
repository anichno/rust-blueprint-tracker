use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ClientMessage {
    pub register: Option<bool>,
    pub login: Option<String>,
    pub learned_bp: Option<i32>,
    pub forgot_bp: Option<i32>,
    pub clear_bp: Option<bool>,
    pub join_team: Option<String>,
    pub create_team: Option<bool>,
    pub leave_team: Option<bool>,
    pub update_name: Option<String>,
    pub color: Option<String>
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ServerMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_members: Option<Vec<TeamMember>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_bps: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_known_bps: Option<Vec<TeamUserBp>>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TeamMember {
    pub user_name: Option<String>,
    pub user_id: String,
    pub color: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct TeamUserBp {
    pub user_id: String,
    pub bp: i32,
}