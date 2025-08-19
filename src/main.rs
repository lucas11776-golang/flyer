use std::{io::Result};

fn main() -> Result<()> {
    // let mut server = flyer::server("127.0.0.1".to_string(), 9999)?;
    let mut server = flyer::server_tls(
        "127.0.0.1".to_string(),
        9999,
        "host.key".to_owned(),
        "host.cert".to_owned(),
    )?;

    server.router().get("/".to_owned(), |_req, res| {
        return res.html("<h1>Hello World!!!</h1>".to_owned());
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();

    Ok(())
}