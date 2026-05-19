use flyer::{mail::Mail, server};


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

        Mail::new()
            .from(String::from("no-reply@test.com"), Some(String::from("no-reply")))
            .send(vec![String::from("jeo@doe.com")])
            .await
            .unwrap();



        return res.html("<h1>Reset Password Sent!!!</h1>")
    });

    print!("\r\n\r\nRunning server: {}\r\n\r\n", server.address());

    server.listen();
}