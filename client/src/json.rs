use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "cmd", content = "args", rename_all = "UPPERCASE")]
pub enum CommandReqest {
    Authorize {
        client_id: String,
        scopes: Vec<String>,
    },
    Authenticate {
        access_token: String,
    },
    #[serde(rename = "SELECT_VOICE_CHANNEL")]
    SelectVoiceChannel {
        channel_id: String,
        force: bool,
    },
    #[serde(rename = "GET_SELECTED_VOICE_CHANNEL")]
    GetSelectedVoiceChannel {},
    #[serde(rename = "SET_USER_VOICE_SETTINGS")]
    SetUserVoiceSettings {
        user_id: String,
        volume: f64,
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "cmd", content = "data", rename_all = "UPPERCASE")]
pub enum CommandResponse {
    Authorize {
        code: String,
    },
    Authenticate {
        scopes: Vec<String>,
    },
    #[serde(rename = "SELECT_VOICE_CHANNEL")]
    SelectVoiceChannel {
        id: String,
        name: String,
        voice_states: Vec<VoiceStates>,
    },
    #[serde(rename = "GET_SELECTED_VOICE_CHANNEL")]
    GetSelectedVoiceChannel {
        id: String,
        name: String,
        voice_states: Vec<VoiceStates>,
    },
    #[serde(rename = "SET_USER_VOICE_SETTINGS")]
    SetUserVoiceSettings {
        user_id: String,
        volume: u32,
    }
}
#[derive(Debug, Deserialize, Serialize)]
pub struct VoiceStates {
    voice_state: VoiceState,
    pub user: User,
    nick: String,
    volume: f64,
    mute: bool,
    pan: Pan,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoiceState {
    mute: bool,
    deaf: bool,
    self_mute: bool,
    self_deaf: bool,
    suppress: bool,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    username: String,
    discriminator: String,
    bot: bool,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Pan {
    left: f32,
    right: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    pub nonce: Uuid,
    #[serde(flatten)]
    pub command: CommandReqest,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub nonce: Uuid,
    #[serde(flatten)]
    pub command: CommandResponse,
}
