extern crate auth_rocket;

use auth_rocket::net::{ random_string, random_string_with_consonants };

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
