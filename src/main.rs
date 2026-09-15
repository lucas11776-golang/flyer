use std::time::Duration;

use flyer::{
    request::Request, response::Response, server, server_tls, session::local::LocalSession,
};


pub async fn stream_controller(_req: Request, res: Response) -> Response {
    // Will only write headers once when write is called
    let res = res
        .set_header("Transfer-Encoding", "chunked")
        .set_header("Connection", "keep-alive")
        .set_header("Content-Type", "text/plain");

    for i in 1..=5 {
        let data = format!("Data payload chunk #{}\n", i);

        // Format: <HEX_SIZE>\r\n<DATA>\r\n
        let chunk_header = format!("{:X}\r\n", data.len());

        res.write(chunk_header.into()).await.unwrap();
        res.write(data.into()).await.unwrap();
        res.write("\r\n".into()).await.unwrap();
    }

    res
}

pub fn main() {
    // let server: &mut flyer::server::Server = server_tls("127.0.0.1", 9999, "host.key", "host.cert")
    let server: &mut flyer::server::Server = server("127.0.0.1", 9999)
        .view("views")
        .session(LocalSession::new(
            Some("sessions"),
            Duration::from_secs(60 * 60),
        ));

    server.router().get("/", stream_controller);

    server.listen();
}