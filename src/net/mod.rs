pub mod key;
pub mod uri;

use rand;

/// Get random string
///
/// ```
/// use auth_rocket::net::random_string;
///
/// for i in 0..100u8 {
///     let s = random_string(i);
///     assert_eq!(i as usize, s.len());
/// }
/// ```
pub fn random_string(len: u8) -> String {
    random_string_with_consonants(len, None)
}

/// Get random string
///
/// ```
/// use auth_rocket::net::random_string_with_consonants;
///
/// assert_eq!(random_string_with_consonants(2, Some("A")), "AA".to_string());
///
/// ```
pub fn random_string_with_consonants(len: u8, consonants: Option<&str>) -> String {
    let consonants = consonants.unwrap_or("abcdfghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890");

    let mut result = String::new();

    for _ in 0..len {
        result.push(rand::sample(&mut rand::thread_rng(), consonants.chars(), 1)[0]);
    }

    result
}

#[cfg(test)]
mod test {
    use ::net::{ random_string, random_string_with_consonants };

    #[test]
    fn test_random_string() {
        for i in 0..100u8 {
            let s = random_string(i);
            assert_eq!(i as usize, s.len());
        }
    }

    #[test]
    fn test_random_string_with_consonants() {
        assert_eq!(random_string_with_consonants(2, Some("A")), "AA".to_string());
    }
}
