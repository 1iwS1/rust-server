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
pub fn belongs_to_group(u_id: Id, g_id: Id, u_gs: &HashMap<UGId,UGProps>) -> bool
{
    return u_gs.contains_key(&UGId { u_id, g_id });
}

pub fn admins_count(g_id: Id, u_gs: &HashMap<UGId, UGProps>) ->usize
{
    let iter = u_gs.into_iter();
    let collection = iter.filter(|&x| x.0.g_id == g_id && x.1.level == LevelAccess::Admin);
    return collection.count();
}
pub fn is_admin(u_id: Id, g_id: Id, map: &HashMap<UGId, UGProps>) -> bool
{
    map.get(
        &UGId {
            u_id,
            g_id,
        }
    ).unwrap().level == LevelAccess::Admin
}

pub fn get_secret_santa(group: &Vec<Id>) -> HashMap<Id, Id>
{
    let mut secret_santa = HashMap::new();
    let mut change_fist = false;
    let mut first_in_line:Id = 0;
    for u_id in group{
        if change_fist{
            first_in_line = *u_id;
            change_fist = false;
        }
        let mut santa_id = u_id + 1;
        if !group.contains(&santa_id){
            santa_id = first_in_line;
            change_fist = true;
        }
        secret_santa.insert(*u_id, santa_id);
    }
    secret_santa
}