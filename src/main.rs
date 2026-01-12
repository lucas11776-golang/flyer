use flyer::server;
use serde::{Deserialize, Serialize};
use std::net::{TcpListener, UdpSocket};

const DNS_HOST: &'static str = "127.0.0.1";
const DNS_PORT: u16 = 53; // Change use by OS bind to it

#[derive(Serialize, Deserialize)]
struct ApiInfo<'a> {
    info: &'a str,
    version: i32
}

fn main() {
    let mut server = server("127.0.0.1", 80);

    server.router().get("/", async |_req, res| {
        return res.html("<h1>Home Page</h1>");
    });

    server.router().subdomain("api", |router| {
        router.get("/", async  |_req, res| {
            return res.json(&ApiInfo {
                info: "Application details",
                version: 1
            });
        });
    });

    server.router().subdomain("{client}", |router| {
        router.get("/", async |req, res| {
            return res.html(format!("<h1>Client Name {}</h1>", req.parameter("client")).as_str());
        });
    });

    server.router().subdomain("{client}.accounts.{account_id}", |router| {
        router.get("/", async |req, res| {
            return res.html(format!("<h1>Client Name {} Account {}</h1>", req.parameter("client"), req.parameter("account_id")).as_str());
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}


async fn udp() {
    let socket = UdpSocket::bind(format!("{}:{}", DNS_HOST, DNS_PORT)).unwrap();
    let mut buf = [0; 1024];

    loop {
        // DO SOME DNS RESOLVING AND SEND RESPONSE
    }
}

async fn tcp() {
    let listener = TcpListener::bind("127.0.0.1:5354").unwrap();
    println!("TCP Server listening on port 5354");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                tokio::spawn(async move {
                    let mut buf = [0; 1024];
                    // DO SOME DNS RESOLVING AND SEND RESPONSE
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}


/*
MacOs DNS FILE

```dns
nameserver 127.0.0.1
port 53 # Change use by OS bind to it
```
*/

