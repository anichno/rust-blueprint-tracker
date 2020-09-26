#[macro_use] extern crate log;
use std::net::{TcpListener, TcpStream};
use std::{thread, io, path, env};
use dgraph::{make_dgraph, Dgraph};

use tungstenite::{Message, accept, WebSocket};
use tungstenite::handshake::server::{Request, Response};
use serde_json::Result;
use std::io::{Error, ErrorKind};
use std::fs::File;

mod messages;
mod database;

fn get_message(websocket: &mut WebSocket<TcpStream>) -> io::Result<messages::ClientMessage> {
    let msg = websocket.read_message();
    debug!("get_message raw msg: {:?}", msg);
    if let Ok(msg) = msg {
        if msg.is_text() {
            let clientMessage: Result<messages::ClientMessage> = serde_json::from_str(&msg.to_string());
            return match clientMessage {
                Ok(clientMessage) => Ok(clientMessage),
                Err(_) => Err(Error::from(ErrorKind::InvalidInput))
            };
        }
    } else {
        return match msg.unwrap_err() {
            tungstenite::error::Error::AlreadyClosed => Err(Error::from(ErrorKind::BrokenPipe)),
            _ => Err(Error::from(ErrorKind::InvalidInput))
        };
    }
    Err(Error::from(ErrorKind::InvalidInput))
}

fn handle_authenticated_client(mut websocket: WebSocket<TcpStream>, dgraph: Dgraph, user: database::User) {
    // Send initial user/team data dump
    let mut team_id = None;
    let mut team_members = None;
    let mut team_known_bps = None;
    if let Some(mut team) = user.team {
        let team = team.pop().unwrap();
        team_id = Some(team.team_id);
        if let Some(members) = team.team_members {
            let mut team_members_vec = Vec::new();
            let mut team_known_bps_vec = Vec::new();
            for member in members {
                team_members_vec.push(messages::TeamMember{
                    user_name: member.user_name,
                    user_id: member.user_id.clone(),
                    color: member.color
                });
                if let Some(blueprints) = member.blueprints {
                    for blueprint in blueprints {
                        team_known_bps_vec.push(messages::TeamUserBp{
                            user_id: member.user_id.clone(),
                            bp: blueprint.bp_id
                        });
                    }
                }
            }
            team_members = Some(team_members_vec);
            team_known_bps = Some(team_known_bps_vec);
        }
    }

    let mut known_bps = Vec::new();
    if let Some(blueprints) = user.blueprints {
        for blueprint in blueprints {
            known_bps.push(blueprint.bp_id);
        }
    }

    let msg = messages::ServerMessage{
        error: None,
        authentication_id: None,
        authenticated: Some(true),
        user_id: Some(user.user_id),
        user_name: user.user_name,
        color: user.color,
        team_id,
        team_members,
        known_bps: Some(known_bps),
        team_known_bps
    };

    websocket.write_message(Message::from(serde_json::to_string(&msg).unwrap()));

    let auth_id = &user.authentication_id.unwrap();

    'outer: loop {
        let msg = get_message(&mut websocket);
        debug!("{:?}", msg);
        match msg {
            Ok(msg) => {
                info!("{:?}", msg);
                if let Some(learned_bp) = msg.learned_bp {
                    database::user_add_blueprint(&dgraph, auth_id, learned_bp);
                }
                if let Some(forgot_bp) = msg.forgot_bp {
                    database::user_remove_blueprint(&dgraph, auth_id, forgot_bp);
                }
                if let Some(create_team) = msg.create_team {
                    if create_team {
                        database::leave_team(&dgraph, auth_id);
                        let team_id = database::create_team(&dgraph, auth_id);
                        let resp_msg = messages::ServerMessage{
                            team_id: Some(team_id),
                            ..Default::default()
                        };
                        websocket.write_message(Message::from(serde_json::to_string(&resp_msg).unwrap()));
                    }
                }
                if let Some(join_team) = msg.join_team {
                    database::leave_team(&dgraph, auth_id);
                    let suc = database::join_team(&dgraph, auth_id, &join_team);
                    let resp_msg = if suc {
                        messages::ServerMessage{
                            team_id: Some(join_team),
                            ..Default::default()
                        }
                    } else {
                        messages::ServerMessage{
                            error: Some("No Such Team".to_string()),
                            ..Default::default()
                        }
                    };

                    websocket.write_message(Message::from(serde_json::to_string(&resp_msg).unwrap()));
                }
                if let Some(leave_team) = msg.leave_team {
                    if leave_team {
                        database::leave_team(&dgraph, auth_id);
                        let resp_msg = messages::ServerMessage{
                            team_id: Some("".to_string()),
                            ..Default::default()
                        };

                        websocket.write_message(Message::from(serde_json::to_string(&resp_msg).unwrap()));
                    }
                }
                if let Some(new_name) = msg.update_name {
                    database::update_name(&dgraph, auth_id, &new_name);
                    let resp_msg = messages::ServerMessage{
                        user_name: Some(new_name),
                        ..Default::default()
                    };

                    websocket.write_message(Message::from(serde_json::to_string(&resp_msg).unwrap()));
                }

                if let Some(color) = msg.color {
                    database::update_color(&dgraph, auth_id, &color);
                    let resp_msg = messages::ServerMessage{
                        color: Some(color),
                        ..Default::default()
                    };

                    websocket.write_message(Message::from(serde_json::to_string(&resp_msg).unwrap()));
                }

                if let Some(clear_bp) = msg.clear_bp {
                    if clear_bp {
                        database::user_clear_blueprints(&dgraph, auth_id);
                        let resp_msg = messages::ServerMessage{
                            known_bps: Some(Vec::new()),
                            ..Default::default()
                        };

                        websocket.write_message(Message::from(serde_json::to_string(&resp_msg).unwrap()));
                    }
                }
            },
            Err(e) => match e.kind() {
                ErrorKind::BrokenPipe => break 'outer,
                _ => continue
            }
        }
    }
}

fn handle_client(stream: TcpStream) {
    let mut websocket = accept(stream);
    if let Ok(mut websocket) = websocket {
        let msg = get_message(&mut websocket);
        debug!("{:?}", msg);
        if let Ok(clientMessage) = msg {
            let dgraph = make_dgraph!(dgraph::new_dgraph_client("dgraph:9080"));
            info!("{:?}", clientMessage);
            if let Some(_) = clientMessage.register {
                let authentication_id = database::create_new_user(&dgraph);
                let result = messages::ServerMessage{
                    authentication_id: Some(authentication_id),
                    ..Default::default()
                };
                websocket.write_message(Message::from(serde_json::to_string(&result).unwrap()));
            } else if let Some(login) = clientMessage.login {
                let user = database::get_user(&dgraph, &login);
                debug!("{:#?}", user);
                if let Some(user) = user {
                    info!("login: {} accepted", login);
                    handle_authenticated_client(websocket, dgraph, user);
                }
            }
        }
    }
}

fn main() {
    env_logger::init();

    // Setup database
    if let Ok(val) = env::var("TESTING") {
        if val == "init" {
            let dgraph = make_dgraph!(dgraph::new_dgraph_client("dgraph:9080"));
            database::drop_schema(&dgraph);
            database::set_schema(&dgraph);
        }
    } else if !path::Path::new("/dgraph/.schema_ready").exists() {
        let dgraph = make_dgraph!(dgraph::new_dgraph_client("dgraph:9080"));
        database::drop_schema(&dgraph);
        database::set_schema(&dgraph);

        File::create("/dgraph/.schema_ready").expect("Failed to touch schema ready file");
    }

    let server = TcpListener::bind("0.0.0.0:9090").unwrap();
    info!("Listening for connections");
    for stream in server.incoming() {
        if let Ok(stream) = stream {
            thread::spawn(move || handle_client(stream));
        }
    }
}