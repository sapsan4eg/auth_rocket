extern crate redis;
extern crate r2d2;
extern crate r2d2_redis;
extern crate auth_rocket;

use redis::RedisError;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use std::io::{ Error, ErrorKind };

use auth_rocket::redisdb::RedisEntity;
use auth_rocket::{ Entity, UserStatus, Role, AuthError };
use std::collections::HashMap;
use redis::Commands;

#[test]
fn test_redis_db() {
    let pool = connect_pool("redis://127.0.0.1/", true);
    if let Ok(con) = pool.get() {
        con.del::<String, i32>("functional_testsauthorize:increment".to_string()).unwrap();
    }
    let entity: RedisEntity = RedisEntity::new(&pool, "functional_tests".to_string());
    functional_tests(&entity);
}

fn functional_tests(entity: &Entity) {
    remove_old_values(entity);

    let mut attrinbutes: HashMap<String, String> = HashMap::new();
    attrinbutes.insert("phone".to_string(), "+79020055555".to_string());

    let user = entity.add_user("Test user", "test@example.com", "qwertyu", attrinbutes.clone()).unwrap();
    assert_eq!(user.status, UserStatus::Created);
    let user = entity.enable_user(user.name.as_str()).unwrap();

    let user_n = entity.get_user_by_name_and_pwd("Test user", "qwertyu").unwrap();
    assert_eq!(user, user_n);
    assert_eq!(user.status, UserStatus::Active);
    assert_eq!(entity.add_token(user.name.as_str(), "just_my_token"), None);
    assert_eq!(entity.get_token(user.name.as_str()).unwrap(), "just_my_token".to_string());
    assert_eq!(entity.get_user_by_token("just_my_token").unwrap(), user);
    assert_eq!(entity.delete_token("just_my_token"), None);
    assert_eq!(entity.get_user_by_token("just_my_token"), Err(AuthError::NotFound));
    assert_eq!(user.attributes, attrinbutes);

    let list = entity.list_users(0, 1_000_000).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(user, list[0]);

    entity.disable_user(user.name.as_str()).unwrap();
    let user = entity.get_user_by_name(user.name.as_str()).unwrap();
    assert_eq!(user.status, UserStatus::Disabled);

    entity.add_user_role(user.name.as_str(), Role::Admins).unwrap();
    let user = entity.get_user_by_id(user.id).unwrap();
    assert_eq!(user.role, Role::Admins);

    entity.add_user_role(user.name.as_str(), Role::Custom("Batman".to_string())).unwrap();
    let user = entity.get_user_by_id(user.id).unwrap();
    assert_eq!(user.role, Role::Custom("Batman".to_string()));

    assert_eq!(entity.delete_user(user.id), None);
    let list = entity.list_users(0, 1_000_000).unwrap();
    assert_eq!(list.len(), 0);

    remove_old_values(entity);
}

fn remove_old_values(entity: &Entity) {
    entity.list_users(0, 1_000_000).map(|list| {
        for user in list {
            println!("Delete user: {:?}", user);
            if entity.delete_user(user.id) != None {
                println!("User not exist: {:?}", user);
            }
        }

        ()
    }).unwrap_or(())
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
