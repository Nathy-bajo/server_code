use std::{collections::HashMap, convert::Infallible, sync::{Arc, RwLock}};
use std::hash::Hash;
use serde::{Serialize, Deserialize};
use tide::{Body, Request, Response};
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

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<RwLock<HashMap<String, User>>>;

#[tokio::main]
async fn main() {
    let mut app = tide::with_state(Arc::new(RwLock::new(init_users())));

    app.at("/login").post(login_handler);
    app.at("/user").post(user_handler);
    app.at("/admin").post(admin_handler);

    app.listen("127.0.0.1:8080").await;
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

pub async fn login_handler(request: Request<Users>) -> tide::Result {
    match users.read() {
        Ok(read_handle) => {
            match read_handle
            .iter()
            .find(|(_uid, user)| user.email == body.email && user.pw == body.pw)
            {
                Some((uid, user)) => {
                    let token = auth::create_jwt(&uid, &Role::from_str(&user.role))
                    .map_err(|e| reject::custom(e))?;
                    Ok(Body::from_json(&LoginResponse { token }).into())
                }
                None => Err(reject::custom(WrongCredentialsError)),
            }
        }
        Err(_) => Err(reject()),
    }
}

pub async fn user_handler(uid: String) -> tide::Result {
    Ok(format!("Hello User {}", uid).into())
}

pub async fn admin_handler(uid: String) -> tide::Result {
    Ok(format!("Hello Admin {}", uid).into())
}
