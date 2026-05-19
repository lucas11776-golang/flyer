use flyer::{mail::{Mail, Mailbox}, server};
use uuid::Uuid;

/*

TODO: Install MailHog for simple mail testing in your `local` environment.

### MacOS/Linux

```sh
mailhog -smtp-bind-addr 127.0.0.1:5555
```

### Windows

```sh
mailhog.exe -smtp-bind-addr 127.0.0.1:5555
```

*/

fn main() {
    let server = server("127.0.0.1", 9999)
        .mailer(String::from("127.0.0.1"), 5555, String::new(), String::new(), false);
    
    server.router().get("/reset-password", async |_req, res| {
        // Send to single email
        Mail::new()
            .from(String::from("no-reply@test.com"), Some(String::from("no-reply")))
            .html(format!("<h1>Password Reset token: {}", Uuid::new_v4()))
            .send(String::from("jeo@doe.com"), Some(String::from("Jeo Deo")))
            .await
            .unwrap();

        // Send to main email
        Mail::new()
            .from(String::from("no-reply@test.com"), Some(String::from("no-reply")))
            .html(format!("<h1>Password Reset token: {}", Uuid::new_v4()))
            .send_to_many(vec![Mailbox::new(String::from("jane@doe.com"), Some(String::from("Jane Deo")))])
            .await
            .unwrap();

        return res.html("<h1>Reset Password Sent!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}