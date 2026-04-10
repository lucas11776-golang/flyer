# TODO LIST

### Working on simplifying FormValidation rules

- 127.0.0.1 is not the same as localhost...

```rust
rules
    .rule("email", vec!["required", "email"])
    .rule("password", vec!["required", "string", "min:5", "max:21", "confirmed"])
    .handle(req, res, next);
```

- Impl file session cleanup to check if file has `FLYER` prefix.
- Refactor session file.
- Write docs for session `Cookie` and `File`.
- Remove old code with new rewrite.


# NEXT VERSION

- Usability is stable must rewrite with tests
- Must add application test
```rust

pub fn setup() {
    // Setup database etc.
}

// TODO: example
pub fn home_page_test() {
    setup();
    
    let app = test(server());
    let req = TestRequest::new()
        .path("dashboard")
        .header("authorization", "xxx");
    let res = app.get(res);

    res.assertOkStatus()
        .assertView("dashboard/index.html");
}
```