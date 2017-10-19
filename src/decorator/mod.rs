use super::{ Entity, User, AuthError, Role, PrivateUser };
use std::collections::HashMap;

pub struct AuthEntity {
    component: Box<Entity>
}

impl AuthEntity {
    pub fn new(component: Box<Entity>) -> Self {
        AuthEntity {
            component: component
        }
    }
}

impl Entity for AuthEntity {

    fn add_user(&self, name: &str, email: &str, password: &str, attributes: HashMap<String, String>) -> Result<User, AuthError> {
        self.component.add_user(name, email, password, attributes)
    }

    fn get_user_by_id(&self, user_id: i32) -> Result<User, AuthError> {
        self.component.get_user_by_id(user_id)
    }

    fn get_user_by_name(&self, username: &str) -> Result<PrivateUser, AuthError> {
        self.component.get_user_by_name(username)
    }

    fn get_user_by_name_and_pwd(&self, username: &str, password: &str) -> Result<User, AuthError> {
        self.component.get_user_by_name_and_pwd(username, password)
    }

    fn delete_user(&self, user_id: i32) -> Option<AuthError> {
        self.component.delete_user(user_id)
    }

    fn list_users(&self, from: isize, count:isize) -> Result<Vec<User>, AuthError> {
        self.component.list_users(from, count)
    }

    fn enable_user(&self, username: &str) -> Result<User, AuthError> {
        self.component.enable_user(username)
    }

    fn disable_user(&self, username: &str) -> Result<User, AuthError> {
        self.component.disable_user(username)
    }

    fn get_token(&self, username: &str) -> Result<String, AuthError> {
        self.component.get_token(username)
    }

    fn add_token(&self, username: &str, token: &str) -> Option<AuthError> {
        self.component.add_token(username, token)
    }

    fn get_user_by_token(&self, token: &str) -> Result<User, AuthError> {
        self.component.get_user_by_token(token)
    }

    fn delete_token(&self, token: &str) -> Option<AuthError> {
        self.component.delete_token(token)
    }

    fn add_user_role(&self, username: &str, role: Role) -> Result<User, AuthError> {
        self.component.add_user_role(username, role)
    }
}
