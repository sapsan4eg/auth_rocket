extern crate auth_rocket;

use auth_rocket::net::key::{ generate_api_key, validate_api_key };

#[test]
fn test_validate_api_key() {
    let key = generate_api_key("secret word").unwrap();
    assert_eq!(validate_api_key(key.as_str(), "secret Word"), false);
    assert_eq!(validate_api_key(key.as_str(), "secret word"), true);
}
