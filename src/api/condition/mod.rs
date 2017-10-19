use rocket::request::{FromForm, FormItems};

pub struct LimitOffset {
    limit: isize,
    offset: isize
}

impl LimitOffset {
    pub fn new(limit: isize, offset: isize) -> Self {
        LimitOffset {
            limit: limit,
            offset: offset
        }
    }

    pub fn get_limit(&self) -> isize {
        self.limit.clone()
    }

    pub fn get_offset(&self) -> isize {
        self.offset.clone()
    }
}

impl<'f> FromForm<'f> for LimitOffset {
    // In practice, we'd use a more descriptive error type.
    type Error = ();

    fn from_form(items: &mut FormItems<'f>, _: bool) -> Result<LimitOffset, ()> {
        let mut limit = 0;
        let mut offset = 10;

        for (key, value) in items {
            match key.as_str() {
                "limit" => {
                    if let Ok(Ok(l)) = value.url_decode().map(|l| { l.to_string().parse::<isize>()}) {
                        if limit < l {
                            limit = l;
                        }
                    }
                },
                "offset" => {
                    if let Ok(Ok(o)) = value.url_decode().map(|o| { o.to_string().parse::<isize>()}) {
                        offset = o;
                    }
                },
                _ => {

                }
            }
        }

        Ok(LimitOffset::new(limit, offset))
    }
}
