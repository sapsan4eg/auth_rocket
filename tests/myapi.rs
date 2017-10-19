extern crate redis;
extern crate r2d2;
extern crate r2d2_redis;
extern crate auth_rocket;
extern crate rocket;
extern crate serde_json;

use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use auth_rocket::redisdb::RedisEntity;
use auth_rocket::{ api, PrivateKey, AuthEntity, Role, Entity };
use rocket::local::Client;
use rocket::http::{ Status, Header, ContentType };
use serde_json::{Value};
use std::collections::HashMap;
use redis::Commands;

#[test]
fn test_redis_api() {
    let pool = connect_pool("redis://127.0.0.1/", true);
    if let Ok(con) = pool.get() {
        con.del::<String, i32>("functional_testsauthorize:increment".to_string()).unwrap();
    }

    let redis = RedisEntity::new(&pool, "functional_tests".to_string());

    redis.add_user("admin", "test@example.com", "qwertyu", HashMap::new()).unwrap();
    redis.enable_user("admin").unwrap();
    redis.add_user_role("admin", Role::Admins).unwrap();

    let rocket = rocket::ignite()
        .mount("/api/", api::get_user_routes())
        .manage(PrivateKey::new("there the test".to_string()))
        .manage(AuthEntity::new(Box::new(redis)))
    ;

    let client = Client::new(rocket).expect("valid rocket instance");
    tests(&client);
}

fn tests(client: &Client) {
    sign_up(&client);
    let token: String = sign_in(&client, "test_user", "test_password");
    user_get(&client, token.clone());
    user_get_redirect(&client, token.clone());
    user_get_un_authorize(&client);

    let admin_token: String = sign_in(&client, "admin", "qwertyu");
    user_get(&client, admin_token.clone());

    get_list_users(&client, admin_token.clone());
    get_list_users_un_authorize(&client, token.clone());
    get_list_users_with_limits(&client, admin_token.clone());
}

fn sign_up(client: &Client) {
    let mut request = client
        .post("/api/users/sign_up/")
        .body("{\"username\":\"test_user\",\"email\":\"test@ya.ru\",\"password\":\"test_password\",\"re_password\":\"test_password\",\"attributes\":{\"phone\":\"+79025555555\"}}")
    ;

    request.add_header(Header::new("Content-type", "application/json"));
    request.add_header(Header::new("Accept", "application/json"));

    let mut response = request.dispatch();

    let created_body = response.body_string();

    assert_eq!(response.status(), Status::Created);
    assert_eq!(response.content_type(), Some(ContentType::JSON));
    assert_eq!(response.headers().get_one("Location"), Some("/api/users/user/2"));
    assert_eq!(created_body, Some("{\"data\":{\"attributes\":{\"phone\":\"+79025555555\"},\"email\":\"test@ya.ru\",\"id\":2,\"name\":\"test_user\",\"role\":\"Users\",\"status\":\"Active\"}}".to_string()));
}

fn sign_in(client: &Client, user:&str, pwd: &str) -> String {
    let mut request = client
        .post("/api/users/sign_in/")
        .body(format!("{{\"username\":\"{}\",\"password\":\"{}\"}}", user, pwd))
    ;

    request.add_header(Header::new("Content-type", "application/json"));
    request.add_header(Header::new("Accept", "application/json"));
    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let v: Value = serde_json::from_str(response.body_string().unwrap().as_str()).unwrap();
    v["data"]["token"].to_string()
}

fn user_get(client: &Client, token: String) {
    let mut request = client
        .get("/api/users/user/2/");

    request.add_header(Header::new("Content-type", "application/json"));
    request.add_header(Header::new("Accept", "application/json"));
    request.add_header(Header::new("access_token", token.replace("\"", "")));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);
}

fn user_get_redirect(client: &Client, token: String) {
    let mut request = client
        .get("/api/users/user/4");

    request.add_header(Header::new("Content-type", "application/json"));
    request.add_header(Header::new("Accept", "application/json"));
    request.add_header(Header::new("access_token", token.replace("\"", "")));

    let response = request.dispatch();
    assert_eq!(response.status(), Status::Found);
    assert_eq!(response.headers().get_one("Location"), Some("/api/users/user/2/"));
}

fn user_get_un_authorize(client: &Client) {
    let request = client
        .get("/api/users/user/2");
    let response = request.dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

fn get_list_users(client: &Client, token: String) {
    let mut request = client
        .get("/api/users/list");

    request.add_header(Header::new("Content-type", "application/json"));
    request.add_header(Header::new("Accept", "application/json"));
    request.add_header(Header::new("access_token", token.replace("\"", "")));

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("{\"data\":[{\"attributes\":{},\"email\":\"test@example.com\",\"id\":1,\"name\":\"admin\",\"role\":\"Admins\",\"status\":\"Active\"},{\"attributes\":{\"phone\":\"+79025555555\"},\"email\":\"test@ya.ru\",\"id\":2,\"name\":\"test_user\",\"role\":\"Users\",\"status\":\"Active\"}]}".to_string()));

}

fn get_list_users_with_limits(client: &Client, token: String) {
    let mut request = client
        .get("/api/users/list?limit=1");

    request.add_header(Header::new("Content-type", "application/json"));
    request.add_header(Header::new("Accept", "application/json"));
    request.add_header(Header::new("access_token", token.replace("\"", "")));

    let mut response = request.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.body_string(), Some("{\"data\":[{\"attributes\":{\"phone\":\"+79025555555\"},\"email\":\"test@ya.ru\",\"id\":2,\"name\":\"test_user\",\"role\":\"Users\",\"status\":\"Active\"}]}".to_string()));

}

fn get_list_users_un_authorize(client: &Client, token: String) {
    let mut request = client
        .get("/api/users/list");
    request.add_header(Header::new("Content-type", "application/json"));
    request.add_header(Header::new("Accept", "application/json"));
    request.add_header(Header::new("access_token", token.replace("\"", "")));

    let response = request.dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

fn connect_pool(connect_str: &str, reconnect: bool) -> Pool<RedisConnectionManager> {
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
