use std::io::Result;
use serde::{Deserialize, Serialize};

mod flyer;

#[derive(Serialize, Deserialize)]
pub struct User {
    first_name: String,
    last_name: String,
    email: String,
}

fn _main() -> Result<()> {
    let mut serve = flyer::server("127.0.0.1".to_string(), 9999)?;

    serve.router().get("/".to_owned(), |req, res| {
        return res.body("<h1>Hello World!!!</h1>".as_bytes());
    });

    serve.router().get("/api/users/1".to_owned(), |req, res| {
        return res.json(&User{
            first_name: "Themba Lucas".to_owned(),
            last_name: "Ngubeni".to_owned(),
            email: "thembangubeni04gmail.com".to_owned()
        });
    });

    serve.listen();

    Ok(())
}
