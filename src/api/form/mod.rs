use std::collections::HashMap;

#[derive(Deserialize)]
pub struct SignIn {
    pub username: String,
    pub password: String
}

#[derive(Deserialize)]
pub struct SignUp {
    pub username: String,
    pub email: String,
    pub password: String,
    pub re_password: String,
    pub attributes: HashMap<String, String>
}
