use flyer::{server};

fn main() {
    // let server = server_tls("127.0.0.1", 8080, "host.key", "host.cert");
    let server = server("127.0.0.1", 9999);

    server.router().get("/", async |_req, res| {
        return res.html("<h1>Hello World</h1>");
    });

    server.router().group("api", |router| {
        router.get("/", async |_req, res| {
            return res.html("<h1>Hello World</h1>")
        });
        router.group("users", |router| {
            router.get("/", async |_req, res| {
                return res
            });
            router.group("{id}", |router| {
                router.get("/", async |_req, res| {
                    return res
                });
            });
        });
    }).middleware(async |_req, res, next| {
        return next.handle(res)
    });

    server.router().ws("/", async |_req, _ws| {
        println!("WEBSOCKET ROUTE CALLBACK");
    });

    server.listen();
}