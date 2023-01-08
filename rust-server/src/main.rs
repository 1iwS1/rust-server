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



        app.listen("127.0.0.1:8080").await
    };
    futures::executor::block_on(f)
}
