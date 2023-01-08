pub use std::collections::HashMap;
pub use std::sync::{Arc, Mutex};
pub use tide::{Request, Response};
pub use serde_json::{Value, json, Map};

#[derive(PartialEq,Eq, Clone)]
pub enum LevelAccess
{
    User,
    Admin,
}

type Id = u32;

#[derive(Eq, Hash, PartialEq, Clone, serde::Serialize)]
pub struct UGId
{
    pub u_id: Id,
    pub g_id: Id,
}
#[derive(Clone)]
pub struct UGProps
{
    pub level: LevelAccess,
    pub santa_id: Id,
}
impl UGProps {
    pub fn new(level: LevelAccess) -> UGProps {
        UGProps {
            level,
            santa_id: 0,
        }
    }
}

pub struct DataBase
{
    pub users: HashMap<Id, String>,
    pub u_max_id: Id,
    pub groups: HashMap<Id, bool>,
    pub g_max_id: Id,
    pub u_gs: HashMap<UGId, UGProps>,
}

pub fn get_field<T>(object: &Map<String, Value>, key: &str) -> T
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    object.get(key).unwrap().as_str().unwrap().parse().unwrap()
}

pub fn resp_data(value: Value) -> Response
{
    Response::builder(200)
        .body(tide::Body::from_json(&value).unwrap())
        .build()
}

pub fn resp_empty() -> Response
{
    Response::builder(200).build()
}

pub fn resp_error(msg: &str) -> Response
{
    Response::builder(400)
        .body(tide::Body::from_json(&json!({"error": msg})).unwrap())
        .build()
}

pub fn new_user(input_obj: &Map<String, Value>, state: &Arc<Mutex<DataBase>>) -> Response
{
    let name: String = get_field(input_obj, "name");
    if name.len() > 0
    {
        let mut guard = state.lock().unwrap();
        let id = guard.u_max_id;
        guard.users.insert(id, name);
        guard.u_max_id += 1;

        resp_data(json!({"id": id}))
    }
    else
    {
        resp_error("bad name")
    }
}
