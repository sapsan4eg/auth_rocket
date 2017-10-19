#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate r2d2;
extern crate crypto;
extern crate chrono;
extern crate rand;
#[macro_use]
extern crate rocket_contrib;

#[cfg(feature="with-redis")] extern crate redis;
#[cfg(feature="with-redis")] extern crate r2d2_redis;
#[cfg(feature="with-redis")] pub mod redisdb;

mod decorator;
pub mod limitation;
pub mod net;
pub mod api;

use std::fmt;
use std::error::Error;
use std::str::FromStr;
use std::collections::HashMap;
use std::convert::From;

pub use decorator::AuthEntity;
pub use net::key::{ generate_api_key, PrivateKey };
pub use limitation::user::{ AuthorizedUser, AdminUser, user_from_request };

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum UserStatus {
    Created,
    Active,
    Disabled,
    Unknown
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            UserStatus::Created  => "created",
            UserStatus::Active   => "active",
            UserStatus::Unknown  => "unknown",
            UserStatus::Disabled => "disabled",
        })
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Role {
    Users,
    Admins,
    Custom(String),
}

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "users"  => Role::Users,
            "admins" => Role::Admins,
            _ => Role::Custom(s.to_string())
        })
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Role::Users => "users",
            Role::Admins => "admins",
            Role::Custom(ref role) => role,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct PrivateUser {
    pub id: i32,
    pub name: String,
    pub email: String,
    password: String,
    pub status: UserStatus,
    pub role: Role,
    pub attributes: HashMap<String, String>
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub status: UserStatus,
    pub role: Role,
    pub attributes: HashMap<String, String>
}

impl From<PrivateUser> for User {
    fn from(user: PrivateUser) -> Self {
        User {
            id: user.id,
            name: user.name,
            email: user.email,
            status: user.status,
            role: user.role,
            attributes: user.attributes
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AuthError {
    /// The error thrown by entity  if user with same name exists
    DuplicateUsername,
    /// The error thrown by entity if cannot found in DB
    NotFound,
    /// The error thrown by entity if have some error with DB
    IOError,
    /// The error thrown by entity if user don't have permissions
    AccessDenied,
    /// The error thrown by entity if user not active
    NotActive,
    /// The error thrown by entity if user is disabled
    DisabledUser
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description())
    }
}

impl Error for AuthError {
    fn description(&self) -> &str {
        match *self {
            AuthError::DuplicateUsername => "User with that name already exists",
            AuthError::NotFound => "Cannot find user with that parameters",
            AuthError::IOError => "Some problem with DB",
            AuthError::AccessDenied => "You don't have permissions to that resource",
            AuthError::NotActive => "Sorry, but this user is not activated",
            AuthError::DisabledUser => "Sorry, but this user is disabled",
        }
    }
}

pub trait Entity: Send + Sync + 'static {
    fn add_user(&self, name: &str, email: &str, password: &str, attributes: HashMap<String, String>) -> Result<User, AuthError>;
    fn get_user_by_id(&self, user_id: i32) -> Result<User, AuthError>;
    fn get_user_by_name(&self, username: &str) -> Result<PrivateUser, AuthError>;
    fn get_user_by_name_and_pwd(&self, username: &str, password: &str) -> Result<User, AuthError>;
    fn delete_user(&self, user_id: i32) -> Option<AuthError>;
    fn list_users(&self, from: isize, count:isize) -> Result<Vec<User>, AuthError>;
    fn enable_user(&self, username: &str) -> Result<User, AuthError>;
    fn disable_user(&self, username: &str) -> Result<User, AuthError>;
    fn get_token(&self, username: &str) -> Result<String, AuthError>;
    fn add_token(&self, username: &str, token: &str) -> Option<AuthError>;
    fn get_user_by_token(&self, token: &str) -> Result<User, AuthError>;
    fn delete_token(&self, token: &str) -> Option<AuthError>;
    fn add_user_role(&self, username: &str, role: Role) -> Result<User, AuthError>;
}
