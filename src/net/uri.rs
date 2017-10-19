use rocket::request::{self, Request, FromRequest};
use rocket::Outcome;
use std::string::ToString;

pub struct RequestedUriString(String);

impl<'a, 'r> FromRequest<'a, 'r> for RequestedUriString {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<RequestedUriString, ()> {
        Outcome::Success(RequestedUriString(request.uri().as_str().to_string()))
    }
}

impl ToString for RequestedUriString {
    fn to_string(&self) -> String {
        let mut s: String = self.0.clone();
        let mut s: String = s.drain(..self.0.find('?').unwrap_or(self.0.len())).collect();

        if s.rfind("/") != Some(s.len() - 1) {
            s.push('/');
        }

        s
    }
}
