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


        app.listen("127.0.0.1:8080").await
    };
    futures::executor::block_on(f)
}
