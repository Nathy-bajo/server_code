use std::{collections::HashMap, convert::Infallible, sync::{Arc, RwLock}};
use serde::{Serialize, Deserialize};
use warp::{Filter, Rejection, Reply, reject, reply};
use crate::error::Error::WrongCredentialsError;
use crate::auth::{Role, with_auth};


mod error;
mod auth;

#[derive(Clone)]
pub struct User {
    pub uid: String,
    pub email: String,
    pub pw: String,
    pub role: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub pw: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}




type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<RwLock<HashMap<String, User>>>;

#[tokio::main]
async fn main() {
    let users = Arc::new(RwLock::new(init_users()));

    let login_route = warp::path!("login")
    .and(warp::post())
    .and(with_users(users.clone()))
    .and(warp::body::json::<LoginRequest>())
    .and_then(login_handler);

    let user_route = warp::path!("user")
    .and(with_auth(Role::User))
    .and_then(user_handler);

    let admin_route = warp::path!("admin")
    .and(with_auth(Role::Admin))
    .and_then(admin_handler);

    let routes = login_route 
    .or(user_route)
    .or(admin_route)
    .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn init_users() -> HashMap<String, User> {
    let mut map = HashMap::new();
    map.insert(
        String::from("1"),
        User {
            uid: String::from("1"),
            email: String::from("user@userland.com"),
            pw: String::from("1234"),
            role: String::from("User"),
        },
    );
    map.insert(
        String::from("2"),
        User {
            uid: String::from("2"),
            email: String::from("admin@adminaty.com"),
            pw: String::from("4321"),
            role: String::from("Admin"),
        },
    );
    map
}

pub async fn login_handler( users: Users, body: LoginRequest) -> WebResult<impl Reply> {
    match users.read() {
        Ok(read_handle) => {
            match read_handle
            .iter()
            .find(|(_uid, user)| user.email == body.email && user.pw == body.pw)
            {
                Some((uid, user)) => {
                    let token = auth::create_jwt(&uid, &Role::from_str(&user.role))
                    .map_err(|e| reject::custom(e))?;
                    Ok(reply::json(&LoginResponse { token }))
                }
                None => Err(reject::custom(WrongCredentialsError)),
            }
        }
        Err(_) => Err(reject()),
    }
}

pub async fn user_handler(uid: String) -> WebResult<impl Reply> {
    Ok(format!("Hello User {}", uid))
}

pub async fn admin_handler(uid: String) -> WebResult<impl Reply> {
    Ok(format!("Hello Admin {}", uid))
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}
