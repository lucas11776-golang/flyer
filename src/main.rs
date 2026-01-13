use flyer::server;
use serde::{Deserialize, Serialize};
use tokio::join;
use tokio::net::{TcpListener, UdpSocket};

const DNS_HOST: &'static str = "127.0.0.1";
const DNS_PORT: u16 = 5354;

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

    server.init(async || {
        println!("STARTING DNS SERVER");

        // join!(udp(), tcp());
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}


async fn udp() {
    let socket = UdpSocket::bind(format!("{}:{}", DNS_HOST, DNS_PORT)).await.unwrap();

    loop {
        let mut buf = [0u8; 4096];
        let recv_result = socket.recv_from(&mut buf).await;

        if recv_result.is_err() {
            continue;
        }
    }
}

async fn tcp() {
    let listener = TcpListener::bind(format!("{}:{}", DNS_HOST, DNS_PORT)).await.unwrap();

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(async move {

        });
    }
}
