# Refactor websocket

- Move websocket logic to http1_websocket handler.
- Cleanup websocket http1.



# WORKING ON MAKING MIDDLEWARE ASYNC

- Must make middleware async because DB call may occur of async operation maybe wanted to verify request...


# ************** HTTP3 HANDLER REQUEST NOTE **************

- Request body does not have data find way to get data.


# HTTP

* Add Route Subdomain...
* Request   
* Response
* Middleware
* Router
* Controller


# Handler errors when doing final refactor...
- TCP connection break etc...


# Find find way to run two server in non block way

```rust
fn block_main_thread() {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone: Arc<AtomicBool> = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    }).unwrap();

    while running.load(Ordering::SeqCst) {}
}
```


# HTTP3 websocket

- Check how websocket work in HTTP3


# Remove package
- Now all packages are used remove unused packages.