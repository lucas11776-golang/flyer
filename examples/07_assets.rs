use std::time::Duration;

use flyer::{hooks::assets::AssetsHook, server, view::ViewData};

/// Static Assets Example
///
/// This example demonstrates:
/// - Configuring the assets directory
/// - Serving static files (CSS, JS, Images)
/// - Caching configuration (size and duration)
fn main() {
    // Requires an 'assets' folder and a 'views' folder
    let server = server("127.0.0.1", 9999)
        .hook(AssetsHook::new("assets", Duration::from_secs(60 * 60), 1024 * 10))
        .view("views");

    server.router().get("/", async |_req, res| {
        return res.view("index.html", Some(ViewData::new()));
    });

    println!("Running Server: {}", server.address());

    server.listen();
}
