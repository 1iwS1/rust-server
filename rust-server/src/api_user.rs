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