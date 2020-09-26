use dgraph::{make_dgraph, Dgraph, NQuad};
use std::fs::read_to_string;
use uuid::Uuid;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::{Value, json};
use rand::{thread_rng, Rng};


#[derive(Deserialize, Serialize, Debug, Default)]
pub struct User {
    pub authentication_id: Option<String>,
    pub user_name: Option<String>,
    pub user_id: String,
    #[serde(rename = "~team_members")]
    pub team: Option<Vec<Team>>,
    pub blueprints: Option<Vec<Blueprint>>,
    pub color: Option<String>
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Team {
    pub team_id: String,
    pub team_name: Option<String>,
    pub team_members: Option<Vec<User>>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Blueprint {
    pub bp_id: i32,
    pub bp_date: DateTime<Utc>
}

pub fn drop_schema(dgraph: &Dgraph) {
    let op_drop = dgraph::Operation {
        drop_all: true,
        ..Default::default()
    };

    dgraph.alter(&op_drop).expect("Failed to drop schema.");

    println!("Dropped the schema.");
}

pub fn set_schema(dgraph: &Dgraph) {
    let schema = read_to_string("schema.rdf").expect("Failed to read schema");
    let op_schema = dgraph::Operation {
        schema,
        ..Default::default()
    };

    dgraph.alter(&op_schema).expect("Failed to set schema.");

    println!("Altered schema.");
}


pub fn create_new_user(dgraph: &Dgraph) -> String {
    #[derive(Serialize, Debug)]
    struct NewUser {
        authentication_id: String,
        user_id: String,
        color: String
    }
    let authentication_id = Uuid::new_v4();
    let authentication_id = authentication_id.to_hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_string();
    let user_id = Uuid::new_v4();
    let user_id = user_id.to_hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_string();

    let mut color = [0_u8; 3];
    thread_rng().fill(&mut color[..]);
    let mut color_str = String::new();
    color_str.push('#');
    for byte in &color {
        color_str.push_str(format!("{:02x}", byte).as_str());
    }

    let mut txn = dgraph.new_txn();
    let user = NewUser{authentication_id: authentication_id.clone(), user_id, color: color_str};
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&user).expect("Failed to serialize JSON."));
    txn.mutate(mutation).expect("Failed to create data.");

    // Commit transaction
    txn.commit().expect("Failed to commit mutation");

    authentication_id
}

pub fn get_user(dgraph: &Dgraph, authentication_id: &str) -> Option<User> {
    #[derive(Deserialize, Debug)]
    struct Root {
        user: Vec<User>
    }

    let query = r#"query userQuery($auth_id: string) {
  user(func:eq(authentication_id, $auth_id)) {
    authentication_id
    user_name
    user_id
    color
  	~team_members {
      team_id
      team_name
      team_members @filter(not eq(authentication_id, $auth_id)) {
        user_name
        user_id
        color
        blueprints {
          bp_id
          bp_date
        }
      }
    }
    blueprints {
      bp_id
      bp_date
    }
  }
}"#;

    let mut vars = HashMap::new();
    vars.insert("$auth_id".to_string(), authentication_id.to_string());

    let resp = dgraph
        .new_readonly_txn()
        .query_with_vars(&query, vars)
        .expect("get_user query error");
    debug!("{:?}", String::from_utf8(resp.json.to_vec()));
    let mut root: Root = serde_json::from_slice(&resp.json).expect("Failed to convert slice to JSON.");
    let user = root.user.pop();

    user
}

fn auth_id_to_uid(dgraph: &Dgraph, authentication_id: &str) -> Option<String> {
    let query = r#"query userQuery($auth_id: string) {
  user(func:eq(authentication_id, $auth_id)) {
    uid
  }
}"#;

    let mut vars = HashMap::new();
    vars.insert("$auth_id".to_string(), authentication_id.to_string());

    let resp = dgraph
        .new_readonly_txn()
        .query_with_vars(&query, vars)
        .expect("auth_id_to_uid query error");
    let resp: Value = serde_json::from_slice(&resp.json).expect("Failed to convert slice to JSON.");
    debug!("{:#?}", resp);
    Some(resp["user"][0]["uid"].as_str().unwrap().to_string())
}

pub fn user_add_blueprint(dgraph: &Dgraph, authentication_id: &str, bp_id: i32) {
    #[derive(Serialize, Debug)]
    struct BpAdd {
        uid: String,
        blueprints: Blueprint
    }

    let blueprint = Blueprint{
        bp_id,
        bp_date: Utc::now()
    };

    let uid = auth_id_to_uid(dgraph, authentication_id).unwrap();

    let mut txn = dgraph.new_txn();
    let new_bp = BpAdd{uid, blueprints: blueprint};
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&new_bp).expect("Failed to serialize JSON."));
    txn.mutate(mutation).expect("Failed to create data.");

    // Commit transaction
    txn.commit().expect("Failed to commit mutation");
}

pub fn user_remove_blueprint(dgraph: &Dgraph, authentication_id: &str, bp_id: i32) {
    #[derive(Deserialize, Debug)]
    struct Root {
        blueprints: Vec<UserBp>
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct UserBp {
        uid: String,
        blueprints: Vec<BpUid>
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct BpUid {
        uid: String,
    }

    let query = r#"query bpQuery($auth_id: string, $bp_id: int) {
  blueprints(func:eq(authentication_id, $auth_id))  {
    uid
    blueprints @filter(eq(bp_id, $bp_id)){
      uid
    }
  }
}"#;

    let mut vars = HashMap::new();
    vars.insert("$auth_id".to_string(), authentication_id.to_string());
    vars.insert("$bp_id".to_string(), bp_id.to_string());

    let resp = dgraph
        .new_readonly_txn()
        .query_with_vars(&query, vars)
        .expect("user_remove_blueprint query error");
    let root: Root = serde_json::from_slice(&resp.json).expect("Failed to convert slice to JSON.");

    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();
    mutation.set_delete_json(serde_json::to_vec(&root.blueprints[0]).expect("Failed to serialize JSON."));
    println!("{:?}", mutation);
    txn.mutate(mutation).expect("Failed to delete data.");

    // Commit transaction
    txn.commit().expect("Failed to commit mutation");
}

pub fn user_clear_blueprints(dgraph: &Dgraph, authentication_id: &str) {
    let uid = auth_id_to_uid(dgraph, authentication_id).unwrap();
    let mutate = json!({
    "uid": uid,
    "blueprints": Value::Null
    });
    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();
    mutation.set_delete_json(serde_json::to_vec(&mutate).expect("Failed to serialize JSON."));
    txn.mutate(mutation).expect("Failed to delete data.");

    // Commit transaction
    txn.commit().expect("Failed to commit mutation");
}

pub fn create_team(dgraph: &Dgraph, authentication_id: &str) -> String {
    #[derive(Serialize, Debug)]
    struct NewTeam {
        team_id: String,
        team_members: TeamMember
    }

    #[derive(Serialize, Debug)]
    struct TeamMember {
        uid: String
    }
    let team_id = Uuid::new_v4();
    let team_id = team_id.to_hyphenated().encode_lower(&mut Uuid::encode_buffer()).to_string();

    let uid = auth_id_to_uid(dgraph, authentication_id).unwrap();

    let mut txn = dgraph.new_txn();
    let user = NewTeam{team_id: team_id.clone(), team_members: TeamMember{uid}};
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&user).expect("Failed to serialize JSON."));
    txn.mutate(mutation).expect("Failed to create data.");

    // Commit transaction
    txn.commit().expect("Failed to commit mutation");

    team_id
}

fn get_team_uid(dgraph: &Dgraph, team_id: &str) -> Option<String> {
    #[derive(Deserialize, Debug)]
    struct Root {
        team: Vec<TeamUID>
    }

    #[derive(Deserialize, Debug)]
    struct TeamUID {
        uid: String
    }

    let query = r#"query teamQuery($team_id: string) {
  team(func:eq(team_id, $team_id)) {
    uid
  }
}"#;

    let mut vars = HashMap::new();
    vars.insert("$team_id".to_string(), team_id.to_string());

    let resp = dgraph
        .new_readonly_txn()
        .query_with_vars(&query, vars)
        .expect("user_remove_blueprint query error");
    let root: Root = serde_json::from_slice(&resp.json).expect("Failed to convert slice to JSON.");

    if root.team.len() == 1 {
        Some(root.team[0].uid.clone())
    } else {
        None
    }
}

pub fn join_team(dgraph: &Dgraph, authentication_id: &str, team_id: &str) -> bool {
    let team_uid = get_team_uid(dgraph, team_id);
    if let Some(team_uid) = team_uid {
        let uid = auth_id_to_uid(dgraph, authentication_id).unwrap();

        #[derive(Serialize, Debug)]
        struct Team {
            uid: String,
            team_members: TeamMember
        }

        #[derive(Serialize, Debug)]
        struct TeamMember {
            uid: String
        }

        let mut txn = dgraph.new_txn();
        let user = Team{uid: team_uid, team_members: TeamMember{uid}};
        let mut mutation = dgraph::Mutation::new();
        mutation.set_set_json(serde_json::to_vec(&user).expect("Failed to serialize JSON."));
        txn.mutate(mutation).expect("Failed to create data.");

        // Commit transaction
        txn.commit().expect("Failed to commit mutation");

        true
    } else {
        false
    }
}

// Currently only 1:1 with teams to users, so will remove player from all teams they are joined to
pub fn leave_team(dgraph: &Dgraph, authentication_id: &str) {
    #[derive(Deserialize, Debug)]
    struct Root {
        user: Vec<UserTeams>
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct UserTeams {
        uid: String,
        #[serde(rename = "~team_members")]
        teams: Option<Vec<TeamId>>
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TeamId {
        uid: String,
    }

    let query = r#"query teamQuery($auth_id: string) {
  user(func:eq(authentication_id, $auth_id)) {
    uid
  	~team_members  {
      uid
    }
  }
}"#;

    let mut vars = HashMap::new();
    vars.insert("$auth_id".to_string(), authentication_id.to_string());

    let resp = dgraph
        .new_readonly_txn()
        .query_with_vars(&query, vars)
        .expect("leave_team query error");
    let root: Root = serde_json::from_slice(&resp.json).expect("Failed to convert slice to JSON.");

    println!("{:#?}", root);

    if let Some(teams) = &root.user[0].teams {
        let uid = root.user[0].uid.clone();

        for team_uid in teams.iter() {
            let mutate = json!({
            "uid": team_uid.uid,
            "team_members": {
                "uid": uid.clone()
                }
            });

            let mut txn = dgraph.new_txn();
            let mut mutation = dgraph::Mutation::new();
            mutation.set_delete_json(serde_json::to_vec(&mutate).expect("Failed to serialize json"));
            println!("{:?}", mutation);
            txn.mutate(mutation).expect("Failed to delete data.");

            // Commit transaction
            txn.commit().expect("Failed to commit mutation");
        }
    }
}

pub fn update_name(dgraph: &Dgraph, authentication_id: &str, new_name: &str) {
    let uid = auth_id_to_uid(dgraph, authentication_id).unwrap();

    let mutate = json!({
    "uid": uid,
    "user_name": new_name.to_string()
    });

    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&mutate).expect("Failed to serialize json"));
    println!("{:?}", mutation);
    txn.mutate(mutation).expect("Failed to update name.");

    // Commit transaction
    txn.commit().expect("Failed to commit mutation");
}

pub fn update_color(dgraph: &Dgraph, authentication_id: &str, new_color: &str) {
    let uid = auth_id_to_uid(dgraph, authentication_id).unwrap();

    let mutate = json!({
    "uid": uid,
    "color": new_color.to_string()
    });

    let mut txn = dgraph.new_txn();
    let mut mutation = dgraph::Mutation::new();
    mutation.set_set_json(serde_json::to_vec(&mutate).expect("Failed to serialize json"));
    println!("{:?}", mutation);
    txn.mutate(mutation).expect("Failed to update color.");

    // Commit transaction
    txn.commit().expect("Failed to commit mutation");
}