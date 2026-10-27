///
/// More about `Rust/UI` #[https://rust-ui.com/]
/// 

use flyer::server;
use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct AccountPageProps {
    first_name: &'static str,
    last_name: &'static str,
    email: &'static str
}

#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            h1 { "Welcome Home Page" }
        }
    }
}

#[allow(non_snake_case)]
pub fn AccountPage(account: AccountPageProps) -> Element {
    rsx! {
        div {
            h1 { "Fullname: {account.first_name} {account.last_name}" }
            h1 { "Email: {account.email}" }
        }
    }
}

fn main() {
    let server = server("127.0.0.1", 9999);

    server.router().group("/", |router| {
        router.get("/", async |_req, res| {
            res.component(HomePage)
        });
        router.get("/account", async |_req, res| {
            res.component_with_props(AccountPage, AccountPageProps{
                first_name: "Jeo",
                last_name: "Deo",
                email: "jeo.deo@gmail.com",
            })
        });
    });

    println!("\r\n\r\nRunning Server: {}", server.address());

    server.listen();
}
