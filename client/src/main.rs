#![feature(const_fn)]
#![feature(const_fn_transmute)]
#[macro_use]
extern crate maplit;
mod config;
mod discord;
mod json;
mod map;
use config::Config;
use discord::Discord;
use json::{CommandReqest, CommandResponse, Request, Response, VoiceStates};
use map::Map;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error, path::Path};
use uuid::Uuid;
use zmq;

fn main() {
    // Send panics to sentry
    let _guard = sentry::init("https://a427ecf6d5f849fa88f5d6da166baaa6@o155156.ingest.sentry.io/5557327");

    let mut state = State::new().unwrap();
    let map = Map::new();

    let select_voice_channel = Request {
        nonce: Uuid::new_v4(),
        command: CommandReqest::SelectVoiceChannel {
            channel_id: state.config.channel_id.clone(),
            force: true,
        },
    };

    let select_voice_channel = serde_json::to_string(&select_voice_channel).unwrap();
    let response = state.discord.command(&select_voice_channel).unwrap();
    let response: Response = serde_json::from_str(&response).unwrap();
    if let CommandResponse::SelectVoiceChannel {
        id,
        name,
        voice_states,
    } = response.command
    {
        state.voice_states = voice_states;
    } else {
        panic!("User set invalid");
    }

    let ctx = zmq::Context::new();
    let s = ctx.socket(zmq::SUB).unwrap();
    s.connect("tcp://127.0.0.1:40545").unwrap();
    s.set_subscribe(b"").unwrap();

    let mut mcid_dcid: HashMap<String, String> = HashMap::new();
    let mut dcid_boost: HashMap<String, u32> = HashMap::new();
    for user in state.config.users.iter() {
        mcid_dcid.insert(user.minecraft_id.clone(), user.discord_id.clone());
        dcid_boost.insert(user.discord_id.clone(), user.boost);
    }

    let mut counter = 0;
    loop {
        let mut users: HashMap<String, f64> = HashMap::new();
        for user in state.voice_states.iter() {
            users.insert(user.user.id.clone(), 0.0);
        }

        loop {
            let msg = s.recv_string(0).unwrap().unwrap();
            println!("msg {}", msg);
            if &msg == "frame" {
                break;
            } else {
                let parts: Vec<&str> = msg.split_whitespace().collect();
                let mcid = *parts.get(0).unwrap();
                let dist = *parts.get(1).unwrap();
                let dist: u32 = dist.parse().unwrap();
                let dcid = mcid_dcid.get(mcid);
                println!("{:#?}", dcid);
                match dcid {
                    Some(dcid) => {
                        let boost = dcid_boost.get(dcid).unwrap();
                        let vol = convert_distance(&dist, &boost);
                        let vol = map.get(&vol);
                        users.insert(dcid.clone(), vol);
                    }
                    None => {}
                }
            }
        }

        println!("{:#?}", users);

        for (user, vol) in users {
            let command = Request {
                nonce: Uuid::new_v4(),
                command: CommandReqest::SetUserVoiceSettings {
                    user_id: user,
                    volume: vol,
                },
            };

            let _ = state
                .discord
                .command(&serde_json::to_string(&command).unwrap())
                .unwrap();
        }

        if counter == 4 {
            let get_voice_channel = Request {
                nonce: Uuid::new_v4(),
                command: CommandReqest::GetSelectedVoiceChannel {},
            };

            let get_voice_channel = serde_json::to_string(&get_voice_channel).unwrap();
            let response = state.discord.command(&get_voice_channel).unwrap();
            let response: Response = serde_json::from_str(&response).unwrap();
            if let CommandResponse::GetSelectedVoiceChannel {
                id,
                name,
                voice_states,
            } = response.command
            {
                println!("{:#?}", voice_states);
                state.voice_states = voice_states;
            } else {
                panic!("User set invalid");
            }

            counter = 0;
        } else {
            counter += 1;
        }
    }
}

fn convert_distance(distance: &u32, boost: &u32) -> u32 {
    let distance = *distance as f64;
    let inter = 0f64.max((64f64 - distance) / 64f64);
    let vol = (inter) * (*boost as f64);
    vol as u32
}

#[derive(Debug)]
struct State {
    discord: Discord,
    voice_states: Vec<VoiceStates>,
    config: Config,
    access_token: String,
}

impl State {
    fn new() -> Result<State, Box<dyn Error>> {
        // Client id is built into the binary
        let client_id = env!("CLIENT_ID");
        let config = Config::new(Path::new("proxvoice.config.yml"))?;
        println!("{:#?}", config);
        let discord = Discord::new(&String::from(client_id))?; //&config.client_id)?;
        println!("{:#?}", discord);
        let mut state = State {
            config,
            discord,
            voice_states: Vec::new(),
            access_token: "".to_string(),
        };

        state.discord.handshake()?;

        let authorize = Request {
            nonce: Uuid::new_v4(),
            command: CommandReqest::Authorize {
                client_id: String::from(client_id), //state.config.client_id.clone(),
                scopes: vec!["rpc".to_string()],
            },
        };
        let authorize = serde_json::to_string(&authorize)?;
        let response = state.discord.command(&authorize)?;
        let response: Response = serde_json::from_str(&response)?;

        let discord_oauth;
        if let CommandResponse::Authorize { ref code } = response.command {
            discord_oauth = DiscordOauth {
                code: code.clone(),
                // Request secret is built into the binary.
                secret: String::from(env!("REQUEST_SECRET")),
            };
        } else {
            panic!("Authorization Invalid");
        }

        let r = ureq::post("http://127.0.0.1:8787")
            .set("Content-Type", "application/json")
            .send_string(&serde_json::to_string(&discord_oauth)?.to_string());
        let r = r.into_string()?;
        let r: Value = serde_json::from_str(&r[..])?;


        match r["access_token"] {
            Value::String(ref token) => {
                state.access_token = token.clone();
            }
            _ => return Err(String::from("Access Token Exchange Failed").into()),
        }

        let authenticate = Request {
            nonce: Uuid::new_v4(),
            command: CommandReqest::Authenticate {
                access_token: state.access_token.clone(),
            },
        };
        let authenticate = serde_json::to_string(&authenticate)?;
        let _ = state.discord.command(&authenticate)?;

        Ok(state)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct DiscordOauth {
    code: String,
    secret: String,
}
