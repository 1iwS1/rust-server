use api_user::*;
pub mod api_user;

fn main() -> Result<(), std::io::Error>
{

    let f = async {
        let data = DataBase
        {
            users: HashMap::new(),
            u_max_id: 0,
            groups: HashMap::new(),
            g_max_id: 0,
            u_gs: HashMap::new(),
        };
        let state = Arc::new(Mutex::new(data));
        let mut app = tide::with_state(state);

        app.at("/users")
            .get(|request: Request<Arc<Mutex<DataBase>>>| async move {
                let guard = request.state().lock().unwrap();
                Ok(json!(guard.users))
            });

        app.at("/groups")
            .get(|request: Request<Arc<Mutex<DataBase>>>| async move {
                let guard = request.state().lock().unwrap();
                Ok(json!(guard.groups))
            });

        app.at("/user/create")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let input_obj = body.as_object().unwrap();
                Ok(new_user(input_obj, request.state()))
            });
        app.at("/group/create")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let creator_id: Id = get_field(object, "creator_id");

                let mut guard = request.state().lock().unwrap();
                Ok(if !guard.users.contains_key(&creator_id)
                {
                    resp_error("no such user")
                }
                else
                {
                    let id = guard.g_max_id;
                    guard.groups.insert(id, false);
                    guard.g_max_id += 1;
                    guard.u_gs.insert(
                        UGId
                        {
                            u_id: creator_id,
                            g_id: id,
                        },
                        UGProps::new(LevelAccess::Admin)
                    );
                    resp_data(json!({"g_id": id}))
                })
            });

        app.at("/group/quit")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let g_id: Id = get_field(object, "g_id");
                let u_id: Id = get_field(object, "u_id");

                let mut guard = request.state().lock().unwrap();
                let user_g_id = UGId{u_id, g_id};
                Ok(match guard.u_gs.get(&user_g_id)
                {
                    None => resp_error("user does not belong to this group"),
                    Some(user_group_props) =>
                        {
                            if user_group_props.level == LevelAccess::Admin && admins_count(g_id, &guard.u_gs) < 2
                            {
                                resp_error("user is only one Admin in this group")
                            }
                            else
                            {
                                if *guard.groups.get(&g_id).unwrap()
                                {
                                    resp_error("group is closed")
                                }
                                else
                                {
                                    guard.u_gs.remove(&user_g_id);
                                    resp_empty()
                                }
                            }
                        }
                })
            });

        app.at("/user/update")
            .put(|mut request: Request<Arc<Mutex<DataBase>>>| async move{
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let id : Id = get_field(object, "id");
                let name: String = get_field(object, "name");
                let mut guard = request.state().lock().unwrap();
                return if !guard.users.contains_key(&id)
                {
                    Ok(resp_error("No such id"))
                } else {
                    guard.users.entry(id).and_modify(|k| *k = name);
                    Ok(resp_empty())
                }
            });

        app.at("/group/unadmin")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let admin_id = get_field(object, "admin_id");
                let g_id = get_field(object, "g_id");

                let mut guard = request.state().lock().unwrap();
                let user_g_id = UGId{u_id: admin_id, g_id};
                Ok(match guard.u_gs.get(&user_g_id)
                {
                    None => resp_error("user does not belong to this group"),
                    Some(user_group_props) =>
                        {
                            if user_group_props.level != LevelAccess::Admin
                            {
                                resp_error("This user is not an admin.")
                            }
                            else
                            {
                                if count_admins(g_id, &guard.u_gs) < 2
                                {
                                    resp_error("It is impossible to remove the last admin in a group. You can appoint a new admin and repeat or delete the whole group.")
                                }
                                else
                                {
                                    guard.u_gs.get_mut(&user_g_id).unwrap().level = LevelAccess::User;
                                    resp_empty()
                                }
                            }
                        }
                })
            });
        app.at("/group/delete")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let admin_id = get_field(object, "admin_id");
                let g_id = get_field(object, "g_id");

                let mut guard = request.state().lock().unwrap();
                Ok(match guard.u_gs.get(&UGId{u_id: admin_id, g_id})
                {
                    None => resp_error("user does not belong to this group"),
                    Some(user_group_props) =>
                        {
                            if user_group_props.level != LevelAccess::Admin
                            {
                                resp_error("This user is not an admin.")
                            }
                            else
                            {
                                // Before delete group, we need to delete all users from this group
                                guard.u_gs.retain(|user_g_id, _|
                                    {
                                        user_g_id.g_id != g_id
                                    });
                                guard.groups.remove(&g_id);
                                resp_empty()
                            }
                        }
                }
                )});

        app.at("/group/target_by_id/:u_id/:g_id")
            .get(|request: Request<Arc<Mutex<DataBase>>>| async move{
                let first_id = request.param("u_id")?;
                let second_id = request.param("g_id")?;
                for c in first_id.chars() {
                    if !c.is_numeric() {
                        return Ok(resp_error("Wrong format user id"));
                    }
                }
                for c in second_id.chars() {
                    if !c.is_numeric() {
                        return Ok(resp_error("Wrong format group id"));
                    }
                }
                let u_id: Id = first_id.parse().unwrap();
                let g_id: Id = second_id.parse().unwrap();

                let guard = request.state().lock().unwrap();
                Ok(match guard.u_gs.get(&UGId{u_id, g_id})
                {
                    None => resp_error("user does not belong to this group"),
                    Some(user_group_props) =>
                        {
                            resp_data(json!({"cysh_for_id": user_group_props.santa_id}))
                        }
                })
            });

        app.at("/group/join")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let value: Value = request.body_json().await.unwrap();
                let object = value.as_object().unwrap();
                let u_id = get_field(object, "u_id");
                let g_id = get_field(object, "g_id");

                let mut guard = request.state().lock().unwrap();
                Ok(match guard.groups.get(&g_id)
                {
                    None => resp_error("no such group"),
                    Some(is_closed) =>
                        {
                            if *is_closed
                            {
                                resp_error("group is closed")
                            }
                            else if !guard.users.contains_key(&u_id)
                            {
                                resp_error("no such user")
                            }
                            else
                            {
                                let user_g_id = UGId{u_id, g_id};
                                if guard.u_gs.contains_key(&user_g_id)
                                {
                                    resp_error("user already in group")
                                }
                                else
                                {
                                    guard.u_gs.insert(user_g_id, UGProps::new(LevelAccess::User));
                                    resp_empty()
                                }
                            }
                        }
                    },
                })
            });
        app.at("/group/make_admin")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let g_id: Id = get_field(object, "g_id");
                let member_id: Id = get_field(object, "member_id");
                let admin_id: Id = get_field(object, "admin_id");

                let mut guard = request.state().lock().unwrap();
                Ok(if !guard.groups.contains_key(&g_id)
                {
                    resp_error("no such group")
                }
                else if !belongs_to_group(member_id, g_id, &guard.u_gs)
                {
                    resp_error("user isn't a member of the group")
                }
                else if is_admin(member_id, g_id, &guard.u_gs)
                {
                    resp_error("user is already an admin")
                }
                else if !is_admin(admin_id, g_id, &guard.u_gs)
                {
                    resp_error("admin_id isn't an actual admin's ID")
                }
                else {
                    guard.u_gs.insert(
                        UGId {
                            u_id: member_id,
                            g_id,
                        },
                        UGProps::new(LevelAccess::Admin),
                    );
                    resp_empty()
                }
                )});

        app.at("/group/secret_santa")
            .post(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let g_id: Id = get_field(object, "g_id");
                let admin_id: Id = get_field(object, "admin_id");

                let mut guard = request.state().lock().unwrap();
                Ok(match guard.u_gs.get(&UGId{u_id: admin_id, g_id})
                {
                    None => resp_error("user does not belong to this group"),
                    Some(user_group_props) =>
                        {
                            if user_group_props.level != LevelAccess::Admin
                            {
                                resp_error("its not admin")
                            }
                            else
                            {
                                *guard.groups.get_mut(&(g_id)).unwrap() = true;
                                let group: Vec<Id> = guard.u_gs.keys().filter_map(|key|
                                    match key.g_id == g_id
                                    {
                                        true => Some(key.u_id),
                                        false => None,
                                    }
                                ).collect();
                                let santas: HashMap<Id, Id> = get_secret_santa(&group);
                                for u_id in group
                                {
                                    guard.u_gs.get_mut(&UGId{u_id, g_id}).unwrap().santa_id = *santas.get(&u_id).unwrap();
                                }
                                resp_empty()
                            }
                        }
                })
            });

        app.at("/user/delete")
            .delete(|mut request: Request<Arc<Mutex<DataBase>>>| async move {
                let body: Value = request.body_json().await?;
                let object = body.as_object().unwrap();
                let u_id = get_field(object, "u_id");
                let mut guard = request.state().lock().unwrap();
                Ok(match guard.users.get(&u_id)
                {
                    None => resp_error("This user does not exist."),
                    Some(_name) =>
                        {
                            if guard.u_gs.len() > 0
                            {
                                let iter1 = guard.u_gs.iter();
                                let iter2 = guard.u_gs.iter();
                                let collection = iter1.filter(|&x| x.0.u_id == u_id);
                                let collect_copy = iter2.filter(|&x| x.0.u_id == u_id);
                                let closed_collect = collection.filter(|&x| guard.groups.get(&x.0.g_id).unwrap() == &true);
                                let free_collect = collect_copy.filter(|&x| guard.groups.get(&x.0.g_id).unwrap() == &false);
                                let mut admin_flag = false;
                                let mut vec:Vec<Id> = Vec::new();
                                let mut delete_vec=Vec::new();
                                for x in free_collect
                                {
                                    if x.1.level == LevelAccess::Admin && count_admins(x.0.g_id, &guard.u_gs) == 1
                                    {
                                        admin_flag=true;
                                        vec.push(x.0.g_id);
                                    }
                                    else
                                    {
                                        delete_vec.push(UGId{u_id, g_id: x.0.g_id});
                                    }
                                }
                                if closed_collect.count() > 0
                                {
                                    for x in delete_vec
                                    {
                                        guard.u_gs.remove(&x);
                                    }
                                    if admin_flag == true
                                    {
                                        let mut string: String="User has closed groups. So he was deleted from opened groups, if he wasn't last admin. User cannot be delete from groups: ".to_string();
                                        for x in vec
                                        {
                                            string+=format!("{0}, ", x).as_str();
                                        }
                                        string+="because of last admin.";
                                        resp_error(string.as_str())
                                    }
                                    else
                                    {
                                        resp_error("User has closed groups. So he was deleted from opened groups.")
                                    }
                                }
                                else
                                {
                                    for x in delete_vec
                                    {
                                        guard.u_gs.remove(&x);
                                    }
                                    if admin_flag == false
                                    {
                                        guard.users.remove(&u_id);
                                        resp_empty()
                                    }
                                    else
                                    {
                                        let mut string: String="User cannot be delete from groups: ".to_string();
                                        for x in vec
                                        {
                                            string+=format!("{0}, ", x).as_str();
                                        }
                                        string+="because he is the last admin in these groups.";
                                        resp_error(string.as_str())
                                    }
                                }
                            }
                            else
                            {
                                guard.users.remove(&u_id);
                                resp_empty()
                            }
                        }
                })
            });

        app.listen("127.0.0.1:8080").await
    };
    futures::executor::block_on(f)
}
