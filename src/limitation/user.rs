use ::{ Entity, User, Role };
use rocket::Outcome;
use rocket::http::Status;
use ::AuthEntity;
use rocket::request::{self, Request, FromRequest, State};
use ::net::key::{ PrivateKey, validate_api_key };

#[derive(Deserialize, Serialize)]
pub struct AuthorizedUser(User);

impl AuthorizedUser {
    pub fn get_user(&self) -> &User {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthorizedUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AuthorizedUser, ()> {
        match user_from_request(request, Vec::new()) {
            Outcome::Success(user) => {
                Outcome::Success(AuthorizedUser(user))
            },
            Outcome::Failure(e) => Outcome::Failure(e),
            _ => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AdminUser(User);

impl<'a, 'r> FromRequest<'a, 'r> for AdminUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AdminUser, ()> {
        match user_from_request(request, vec!(Role::Admins)) {
            Outcome::Success(user) => {
                Outcome::Success(AdminUser(user))
            },
            Outcome::Failure(e) => Outcome::Failure(e),
            _ => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

pub fn user_from_request(request: &Request, role: Vec<Role>) -> request::Outcome<User, ()> {
    let keys: Vec<_> = request.headers().get("access_token").collect();

    if keys.len() != 1 {
        return Outcome::Failure((Status::Unauthorized, ()));
    }

    let header_key = keys[0];

    let key: &PrivateKey = match request.guard::<State<PrivateKey>>() {
        Outcome::Success(key) => key.inner(),
        _ => {
            error!("Not found PrivateKey in request.");
            return Outcome::Failure((Status::InternalServerError, ()));
        }
    };

    if validate_api_key(header_key, key.as_str()) == false {
        return Outcome::Failure((Status::Unauthorized, ()));
    }

    match request.guard::<State<AuthEntity>>() {
        Outcome::Success(entity) => {
            match entity.inner().get_user_by_token(header_key) {
                Ok(u) => {
                    if role.len() == 0 || role.contains(&u.role) {
                        Outcome::Success(u)
                    } else {
                        Outcome::Failure((Status::Unauthorized, ()))
                    }
                },
                _ => Outcome::Failure((Status::Unauthorized, ()))
            }
        },
        _ => Outcome::Failure((Status::Unauthorized, ()))
    }
}
