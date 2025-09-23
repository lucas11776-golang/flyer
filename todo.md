# HTTP

* Add Route Subdomain...
* Request   
* Response
* Middleware
* Router
* Controller


# REMEMBER

** If passing struct to thread they must be thread safe you maybe need to implement Send or Sync to tell rust that the are safe..



# Controller must be async

// pub type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> BoxFuture<'a, &'a mut Response> + Send + Sync;


// TODO: for async function must implement still working on it
// pub fn get<C, F>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
// where
//     for<'b> C: Fn(&'b mut Request, &'b mut Response) -> F + Send + Sync + 'static,
//     F: FutureExt<Output =  Response> + Send + Sync + 'static,