#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate auth_rocket;
extern crate r2d2_redis;
extern crate r2d2;
extern crate redis;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use redis::RedisError;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use std::io::{ Error, ErrorKind };
use auth_rocket::redisdb::RedisEntity;
use rocket::request::{self, Request, FromRequest };
use auth_rocket::{ PrivateKey, AuthEntity, User, Role, user_from_request };
use rocket::Outcome;
use rocket::http::Status;

fn main() {
    let redis = connect_pool("redis://127.0.0.1/", true);
    let redis_entity = RedisEntity::new(&redis, "test_example:".to_string());


    #[derive(Deserialize, Serialize)]
    pub struct BatmanCustomUser(User);

    impl<'a, 'r> FromRequest<'a, 'r> for BatmanCustomUser {
        type Error = ();

        fn from_request(request: &'a Request<'r>) -> request::Outcome<BatmanCustomUser, ()> {
            match user_from_request(request, vec!(Role::Custom("Batman".to_string()))) {
                Outcome::Success(user) => {
                    Outcome::Success(BatmanCustomUser(user))
                },
                Outcome::Failure(e) => Outcome::Failure(e),
                _ => Outcome::Failure((Status::Unauthorized, ()))
            }
        }
    }

    rocket::ignite().mount("/api", routes!(get_user, get_user_bat))
        .manage(PrivateKey::new("my_secret_key".to_string()))
        .manage(AuthEntity::new(Box::new(redis_entity))).launch();

    #[get("/user")]
    pub fn get_user(user: auth_rocket::AuthorizedUser) -> String {
        format!("{}", json!(user).to_string())
    }
    #[get("/user/batman")]
    pub fn get_user_bat(user: BatmanCustomUser) -> String {
        format!("{}", json!(user).to_string())
    }
}

pub fn connect_pool(connect_str: &str, reconnect: bool) -> Pool<RedisConnectionManager> {
    let cache = Default::default();

    match RedisConnectionManager::new(connect_str) {
        Ok(m) => {
            match Pool::new(cache, m) {
                Ok(pool) => {
                    pool
                },
                Err(_) => {
                    match reconnect {
                        true => {
                            connect_pool(connect_str, false)
                        },
                        false => {
                            std::thread::sleep(std::time::Duration::from_millis(10000u64));
                            connect_pool(connect_str, false)
                        }
                    }
                }
            }
        },
        Err(_) => {
            match reconnect {
                true => {
                    connect_pool(connect_str, false)
                },
                false => {
                    std::thread::sleep(std::time::Duration::from_millis(10000u64));
                    connect_pool(connect_str, false)
                }
            }
        }
    }
}

pub fn error(e: &str) -> RedisError {
    RedisError::from(Error::new(ErrorKind::Other, e))
}