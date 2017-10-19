mod condition;
mod form;

use rocket_contrib::Json;
use rocket::request::{ State };
use ::net::key::{ generate_api_key, PrivateKey };
use ::limitation::user::{ AuthorizedUser, AdminUser };
use ::{ AuthEntity, Entity, Role };
use ::api::condition::LimitOffset;
use ::api::form::{ SignIn, SignUp };
use rocket::response::{ status, Redirect };
use rocket::http::Status;
use rocket::Route;
use ::net::uri::RequestedUriString;

#[post("/users/sign_up", format = "application/json", data="<sign_up>")]
pub fn sign_up(entity: State<AuthEntity>, sign_up: Json<SignUp>, uri: RequestedUriString) -> Result<status::Created<Json>, status::Custom<Json>> {

    if sign_up.password != sign_up.re_password {
        return Err(status::Custom(Status::BadRequest, Json(json!({
            "error": ""
        }))))
    }

    match entity.inner().add_user(sign_up.username.as_str(), sign_up.email.as_str(), sign_up.password.as_str(), sign_up.attributes.clone()) {
        Ok(user) => {
            let mut uri_str = uri.to_string();
            uri_str.push_str(format!("{}", user.id).as_str());
            Ok(status::Created(uri_str.replace("sign_up/", "user/"), Some(Json(json!({
                "data": match entity.inner().enable_user(user.name.as_str()) {
                    Ok(u) => u,
                    Err(e) => {
                        error!("{}", e);
                        user
                    }
                }
            })))))
        },
        Err(e) => Err(status::Custom(Status::Conflict, Json(json!({
            "error": format!("{}", e)
        }))))
    }
}

#[post("/users/sign_in", format = "application/json", data="<sign_in>")]
pub fn sign_in(private_key: State<PrivateKey>, entity: State<AuthEntity>, sign_in: Json<SignIn>) -> status::Custom<Json> {

    if let Err(e) = entity.inner().get_user_by_name_and_pwd(sign_in.username.as_str(), sign_in.password.as_str()) {
        return status::Custom(Status::Unauthorized, Json(json!({"error": format!("{}", e)})))
    }

    let token: String = generate_api_key(private_key.inner().as_str()).unwrap();

    if let Some(e) = entity.inner().add_token(sign_in.username.as_str(), token.as_str()) {
        return status::Custom(Status::Unauthorized, Json(json!({"error": format!("{}", e)})))
    }

    status::Custom(Status::Ok,Json(json!({"data": {"token": token}})))
}

/*
#[patch("/users/user/<id>", format = "application/json", data="<new_user>")]
pub fn up_user(entity: State<AuthEntity>, new_user: Json<User>, user: AuthorizedUser, id: i32) -> Result<status::Custom<Json>, status::Custom<Json>> {
    Ok(status::Custom(Status::Ok, Json(json!({
                "data": user.get_user()
            }))))
}*/

#[get("/users/user/<id>", format = "application/json")]
pub fn get_user(user: AuthorizedUser, id: i32, entity: State<AuthEntity>, uri: RequestedUriString) -> Result<status::Custom<Json>, Redirect>  {
    if user.get_user().id == id {
        Ok(status::Custom(Status::Ok, Json(json!({"data": user}))))
    } else if user.get_user().role == Role::Admins {
        match entity.inner().get_user_by_id(id) {
            Ok(u) => Ok(status::Custom(Status::Ok, Json(json!({ "data": u })))),
            Err(e) => Ok(status::Custom(Status::NotFound, Json(json!({ "error": format!("{}", e) }))))
        }
    } else {
        Err(Redirect::found(uri.to_string().replace(format!("{}", id).as_str(), format!("{}", user.get_user().id).as_str()).as_str()))
    }
}

#[get("/users/list", format = "application/json")]
pub fn get_user_list(entity: State<AuthEntity>, user: AdminUser) -> Json {
    Json(json!({"data": entity.list_users(0, 10).unwrap()}))
}

#[get("/users/list?<limit>", format = "application/json")]
pub fn get_user_list_with_limit(entity: State<AuthEntity>, limit: LimitOffset, user: AdminUser) -> Json {
    Json(json!({"data": entity.list_users(limit.get_limit(), limit.get_offset()).unwrap()}))
}

pub fn get_user_routes() -> Vec<Route> {
    routes!( sign_up, get_user, sign_in, get_user_list, get_user_list_with_limit)
}
