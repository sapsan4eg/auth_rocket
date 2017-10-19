use crypto::sha1;
use crypto::digest::Digest;
use ::net::random_string;

pub struct PrivateKey {
    key: String
}

impl PrivateKey {

    pub fn new(key: String) -> Self {
        PrivateKey {
            key: key
        }
    }

    pub fn as_str(&self) -> &str {
        &self.key
    }
}

/// Generate valid api key
///
/// ```
/// use auth_rocket::generate_api_key;
/// use auth_rocket::net::key::validate_api_key;
///
/// let key = generate_api_key("secret word").unwrap();
/// assert!(validate_api_key(key.as_str(), "secret word"));
/// ```
pub fn generate_api_key(secret: &str) -> Option<String> {
    let random = random_string(26);
    let mut sh = sha1::Sha1::new();
    sh.input_str(&format!("{}{}", random, secret));
    let hex = sh.result_str();
    hex.get(0_..6).map(|hex_x| { format!("{}{}", random, hex_x) })
}

/// Validate api key
///
/// ```
/// use auth_rocket::generate_api_key;
/// use auth_rocket::net::key::validate_api_key;
///
/// let key = generate_api_key("secret word").unwrap();
/// assert_eq!(validate_api_key(key.as_str(), "secret word"), true);
/// assert_eq!(validate_api_key(key.as_str(), "secret Word"), false);
/// ```
pub fn validate_api_key(key: &str, secret: &str) -> bool {

    if key.len() != 32 {
        return false
    }

    match key.get(26..) {
        Some(hash) => {
            match key.get(0..26) {
                Some(key_real) => {
                    let st = format!("{}{}", key_real, secret);
                    let mut sh = sha1::Sha1::new();
                    sh.input_str(st.as_str());
                    let hex = sh.result_str();
                    match hex.get(0..6) {
                        Some(hex_x) => {
                            hex_x == hash
                        },
                        _ => false
                    }
                },
                _ => false
            }
        },
        _ => false
    }
}

#[cfg(test)]
mod test {
    use ::net::key::{ generate_api_key, validate_api_key };

    #[test]
    fn test_validate_api_key() {
        let key = generate_api_key("secret word").unwrap();
        assert_eq!(validate_api_key(key.as_str(), "secret Word"), false);
        assert_eq!(validate_api_key(key.as_str(), "secret word"), true);
    }
}
