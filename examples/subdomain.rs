use flyer::server;
use flyer::utils::development::dns;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ApiInfo<'a> {
    info: &'a str,
    version: i32
}

fn main() {
    let server = server("127.0.0.1", 80);

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
        tokio::spawn(async {
            dns::run("tracker.com", "127.0.0.1", 5354);
        });
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}

