use redis::Commands;
use std::collections::HashMap;
use super::{Entity, User, AuthError, Role, UserStatus, PrivateUser};
use std::str::FromStr;
use r2d2::{Pool, PooledConnection};
use r2d2_redis::RedisConnectionManager;
use crypto::md5;
use crypto::digest::Digest;
use chrono::Local;
use std::fmt;
use serde_json;

enum StorageNames {
    Name,
    Id,
    Increment,
    List,
    UserToken,
    TokenToken
}

impl fmt::Display for StorageNames {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            StorageNames::Name => "authorize:users:name:",
            StorageNames::Id => "authorize:users:id:",
            StorageNames::Increment => "authorize:increment",
            StorageNames::List => "authorize:users:list",
            StorageNames::UserToken => "authorize:users:tokens:user:",
            StorageNames::TokenToken => "authorize:users:tokens:token:",
        })
    }
}

pub struct RedisEntity {
    pool: Pool<RedisConnectionManager>,
    prefix: String
}

impl RedisEntity {
    pub fn new(s: &Pool<RedisConnectionManager>, prefix: String) -> RedisEntity {
        RedisEntity {
            pool: s.clone(),
            prefix: prefix
        }
    }

    fn get_conn(&self) -> Option<PooledConnection<RedisConnectionManager>> {
        match self.pool.get() {
            Ok(pool) => Some(pool),
            Err(e) => {
                error!("Cannot get redis pool: {}", e);
                None
            }
        }
    }
}

impl Entity for RedisEntity {
    fn get_user_by_name(&self, username: &str) -> Result<PrivateUser, AuthError> {
        self.get_conn()
            .ok_or(AuthError::IOError)
            .and_then(|con| con.hgetall(format!("{}{}{}", self.prefix, StorageNames::Name, username))
                .ok().ok_or(AuthError::NotFound)
                .and_then(|t: HashMap<String, String>| {
                    match t.len() > 0 {
                        true => Ok(PrivateUser {
                            id: i32::from_str(t.get("id").unwrap_or(&"0".to_string())).unwrap_or(0i32),
                            name: t.get("name").unwrap_or(&"default".to_string()).to_string(),
                            password: t.get("password").unwrap_or(&"default".to_string()).to_string(),
                            status: match u8::from_str(t.get("status").unwrap_or(&"0".to_string())).unwrap_or(0u8) { 0u8 => UserStatus::Created, 1u8 => UserStatus::Active, 2u8 => UserStatus::Disabled, _ => UserStatus::Unknown },
                            email: t.get("email").unwrap_or(&"default".to_string()).to_string(),
                            role: Role::from_str(t.get("role").unwrap_or(&"unknown".to_string())).unwrap_or(Role::Custom("unknown".to_string())),
                            attributes: serde_json::from_str::<HashMap<String, String>>(t.get("attributes").unwrap_or(&"{}".to_string()).as_str()).unwrap_or(HashMap::new())
                        }),
                        false => Err(AuthError::NotFound)
                    }
                })
            )
    }

    fn get_user_by_name_and_pwd(&self, username: &str, password: &str) -> Result<User, AuthError> {
        let mut sh = md5::Md5::new();
        sh.input_str(password);
        self.get_user_by_name(username)
            .and_then(|u| match u.password == sh.result_str() {
                true => Ok(User::from(u)),
                false => Err(AuthError::NotFound)
            })
    }

    fn get_user_by_id(&self, user_id: i32) -> Result<User, AuthError> {
        self.get_conn().ok_or(AuthError::IOError)
            .and_then(|con|
                con.get(format!("{}{}{}", self.prefix, StorageNames::Id, user_id))
                    .ok().ok_or(AuthError::NotFound)
                    .and_then(|username: String| self.get_user_by_name(&username).map(|user| User::from(user)))
            )
    }

    fn add_user(&self, name: &str, email: &str, password: &str, attributes: HashMap<String, String>) -> Result<User, AuthError> {
        self.get_user_by_name(name)
            .and(Err(AuthError::DuplicateUsername))
            .or_else(|e| {
                match e {
                    AuthError::NotFound => {
                        self.get_conn().ok_or(AuthError::IOError)
                        .and_then(|con|
                            con.hincr(format!("{}{}", self.prefix, StorageNames::Increment), "users", 1)
                                .ok().ok_or(AuthError::IOError)
                            .and_then(|id: i32|
                                con.set(format!("{}{}{}", self.prefix, StorageNames::Id, id), name)
                                    .ok().ok_or(AuthError::IOError)
                                    .and_then(|_: bool|
                                        con.zadd(format!("{}{}", self.prefix, StorageNames::List), id, Local::now().timestamp() as i64)
                                            .ok().ok_or(AuthError::IOError)
                                            .and_then(|_: bool| {
                                                let mut sh = md5::Md5::new();
                                                sh.input_str(password);
                                                con.hset_multiple(format!("{}{}{}", self.prefix, StorageNames::Name, name),
                                                     &vec!(("id", id.to_string()),
                                                           ("name", name.to_string()),
                                                           ("email", email.to_string()),
                                                           ("status", "0".to_string()),
                                                           ("password", sh.result_str()),
                                                           ("role", Role::Users.to_string()),
                                                           ("attributes", json!(attributes).to_string())
                                                     )
                                                )
                                                .ok().ok_or(AuthError::IOError)
                                                .and_then(|_: bool| self.get_user_by_name(name).map(|user| User::from(user)))
                                            }
                                        )
                                    )
                            )
                        )
                    },
                    _  => Err(e),
                }
            })
    }

    fn delete_user(&self, user_id: i32) -> Option<AuthError> {
        match self.get_conn() {
            Some(con) => {
                match con.get(format!("{}{}{}", self.prefix, StorageNames::Id, user_id)).ok().map(|u: String| u) {
                   Some(u) => {
                       if let Err(e) = con.del(format!("{}{}{}", self.prefix, StorageNames::Name, u)).map(|n: bool| n) {
                           warn!("cannot delete key ({}{}{}) in redis DB ({})", self.prefix, StorageNames::Name, u, e);
                       }
                   },
                   _ => {
                       warn!("username by key ({}{}{}) not found in redis DB", self.prefix, StorageNames::Id, user_id);
                   }
                }

                if let Err(e) = con.del(format!("{}{}{}", self.prefix, StorageNames::Id, user_id)).map(|n: bool| n) {
                    warn!("cannot delete key ({}{}{}) in redis DB ({})", self.prefix, StorageNames::Id, user_id, e);
                }

                if let Err(e) = con.zrem(format!("{}{}", self.prefix, StorageNames::List), user_id).map(|n: bool| n) {
                    println!("cannot delete key ({}) from list {}{} in redis DB ({})", user_id, self.prefix, StorageNames::List, e);
                }

                None
            },
            _ => {
                Some(AuthError::IOError)
            }
        }
    }

    fn list_users(&self, from: isize, count:isize) -> Result<Vec<User>, AuthError> {
        self.get_conn().ok_or(AuthError::IOError)
            .and_then(|con| {
                con.zrange(format!("{}{}", self.prefix, StorageNames::List), from, count).ok().ok_or(AuthError::NotFound)
                    .and_then(|list: Vec<i32>| {
                        let mut v: Vec<User> = Vec::new();
                        for d in &list {
                            match self.get_user_by_id(d.clone()) {
                                Ok(u) => {
                                    v.push(u);
                                },
                                Err(_) => {
                                    warn!("user from list with id {} not found in redis DB", d);
                                }
                            }
                        }
                        Ok(v)
                    })
            })
    }

    fn add_token(&self, username: &str, token: &str) -> Option<AuthError> {
        self.get_conn()
            .and_then(|con| {
                con.set(format!("{}{}{}", self.prefix, StorageNames::UserToken, username), token)
                    .ok()
                    .and_then(|_: bool|
                        con.expire(format!("{}{}{}", self.prefix, StorageNames::UserToken, username), 3600).ok()
                            .and_then(|_: bool| {
                                con.set(format!("{}{}{}", self.prefix, StorageNames::TokenToken, token), username)
                                    .ok()
                                    .and_then(|_: bool| {
                                        con.expire(format!("{}{}{}", self.prefix, StorageNames::TokenToken, token), 3600)
                                            .ok()
                                            .and_then(|_: bool| {None})

                                    })
                            })
                    )
            })
    }

    fn get_user_by_token(&self, token: &str) -> Result<User, AuthError> {
        self.get_conn()
            .ok_or(AuthError::IOError)
            .and_then(|con| con.get(format!("{}{}{}", self.prefix, StorageNames::TokenToken, token))
                .ok().ok_or(AuthError::NotFound)
                .and_then(|username: String| {
                    self.get_user_by_name(&username).and_then(|u|
                        match u.status == UserStatus::Active {
                            true => match self.add_token(&username, token) {
                                Some(e) => Err(e),
                                _ => Ok(User::from(u))
                            },
                            false => Err(AuthError::NotActive)
                        }
                    )
                })
            )
    }

    fn get_token(&self, username: &str) -> Result<String, AuthError> {
        self.get_conn()
            .ok_or(AuthError::IOError)
            .and_then(|con| con.get(format!("{}{}{}", self.prefix, StorageNames::UserToken, username))
                .ok().ok_or(AuthError::NotFound)
                .and_then(|token: String| {
                    Ok(token)
                })
            )
    }

    fn enable_user(&self, username: &str) -> Result<User, AuthError> {
        self.get_user_by_name(username)
            .and_then(|_| self.get_conn()
            .ok_or(AuthError::IOError)
            .and_then(|con| con.hset(format!("{}{}{}", self.prefix, StorageNames::Name, username), "status", "1")
                .ok().ok_or(AuthError::NotFound)
                .and_then(|_: bool| {
                    self.get_user_by_name(username).map(|u| User::from(u))
                })
            ))
    }

    fn disable_user(&self, username: &str) -> Result<User, AuthError> {
        self.get_user_by_name(username)
            .and_then(|u| self.get_conn()
                .ok_or(AuthError::IOError)
                .and_then(|con| con.hset(format!("{}{}{}", self.prefix, StorageNames::Name, username), "status", "2")
                    .ok().ok_or(AuthError::NotFound)
                    .and_then(|_: bool| {
                        self.get_user_by_name(&u.name).map(|user| User::from(user))
                    })
                ))
    }

    fn delete_token(&self, token: &str) -> Option<AuthError> {
        match self.get_user_by_token(token) {
            Ok(u) => { match self.get_conn().ok_or(AuthError::IOError) {
                Ok(con) => {
                    if let Err(e) = con.del(format!("{}{}{}", self.prefix, StorageNames::UserToken, u.name)).map(|n: bool| n) {
                        warn!("cannot delete key ({}{}{}) in redis DB ({})", self.prefix, StorageNames::UserToken, u.name, e);
                    }

                    if let Err(e) = con.del(format!("{}{}{}", self.prefix, StorageNames::TokenToken, token)).map(|n: bool| n) {
                        warn!("cannot delete key ({}{}{}) in redis DB ({})", self.prefix, StorageNames::TokenToken, token, e);
                    }

                    None
                },
                Err(_) => Some(AuthError::IOError)
            } },
            Err(e) => Some(e)
        }
    }

    fn add_user_role(&self, username: &str, role: Role) -> Result<User, AuthError> {
        self.get_user_by_name(username)
            .and_then(|u| self.get_conn()
                .ok_or(AuthError::IOError)
                .and_then(|con|
                    con.hset(format!("{}{}{}", self.prefix, StorageNames::Name, username), "role", role.to_string())
                    .ok().ok_or(AuthError::IOError)
                    .and_then(|_: bool| {
                        self.get_user_by_name(&u.name).map(|user| User::from(user))
                    })
                ))
    }
}
