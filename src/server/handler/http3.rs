use std::io::Result;

use bytes::Bytes;

use http::Request as HttpRequest;
use h3::server::{
    RequestResolver,
    RequestStream
};
use h3_quinn::BidiStream;

use crate::request::{self, Files, Request};
use crate::response::Response;
use crate::server::HTTP3;
use crate::utils::url::parse_query_params;
use crate::utils::Values;
use crate::HTTP;

pub(crate) struct Handler {
    request: HttpRequest<()>,
    stream: RequestStream<BidiStream<Bytes>, Bytes>
}

impl Handler {
    pub fn new(request: HttpRequest<()>, stream: RequestStream<BidiStream<Bytes>, Bytes>) -> Self {
        return Self {
            request: request,
            stream: stream
        }
    }

    // pub async fn handle(&mut self) -> Result<Request> {
    //     let (req, stream) = self.resolver.resolve_request().await.unwrap();
        

    //     let req: http::Request<()> = req;

    //     let mut headers = Values::new();

    //     for (k, v) in req.headers() {
    //         headers.insert(k.to_string(), v.to_str().unwrap().to_string());
    //     }

    //     let host = headers
    //         .get("host")
    //         .cloned()
    //         .or_else(|| headers.get(":authority").cloned())
    //         .unwrap_or_default();

    //     let request = Request{
    //         ip: "127.0.0.1".to_owned(),
    //         host: host.to_string(),
    //         method: req.method().to_string(),
    //         path: req.uri().path().to_string(),
    //         parameters: Values::new(),
    //         query: parse_query_params(req.uri().query().unwrap_or(""))?,
    //         protocol: HTTP3.to_string(),
    //         headers: headers,
    //         body: vec![],
    //         values: Values::new(),
    //         files: Files::new(),
    //     };

    //     Ok((stream, request))
    // }


    // pub async fn write(self, stream: RequestStream<BidiStream<Bytes>, Bytes>, response: &mut Response) -> Result<()> {


    //     Ok(())
    // }
}



// pub struct Handler { }

// impl <'a>Handler {
//     pub async fn handle(http: &'a mut HTTP<'a>, resolver: RequestResolver<h3_quinn::Connection, Bytes>) -> Result<()> {
//         let (req, stream) = resolver.resolve_request().await.unwrap();
//         let mut headers = Values::new();

//         for (k, v) in req.headers() {
//             headers.insert(k.to_string(), v.to_str().unwrap().to_string());
//         }

//         let host = headers
//             .get("host")
//             .cloned()
//             .or_else(|| headers.get(":authority").cloned())
//             .unwrap_or_default();

//         let mut request= Request{
//             ip: "127.0.0.1".to_owned(),
//             host: host.to_string(),
//             method: req.method().to_string(),
//             path: req.uri().path().to_string(),
//             parameters: Values::new(),
//             query: parse_query_params(req.uri().query().unwrap_or(""))?,
//             protocol: HTTP3.to_string(),
//             headers: headers,
//             body: vec![],
//             values: Values::new(),
//             files: Files::new(),
//         };

//         // TODO: move
//         // Handler::handle_request(http, &mut request, stream).await?;

//         Ok(())
//     }

//     async fn handle_request<'s>(http: &'a mut HTTP<'a>, req: &'a mut Request, mut stream: RequestStream<BidiStream<Bytes>, Bytes>) -> Result<()> {
//         // let mut res = new_response();
//         // let res = RequestHandler::web(http, req, &mut res).await.unwrap();
//         // let mut builder = http::Response::builder()
//         //     .status(res.status_code)
//         //     .header("content-length", format!("{}", res.body.len()));

//         // for (k, v) in &mut res.headers {
//         //     builder = builder.header(k.clone(), v.clone());
//         // }

//         // let response = builder.body(()).unwrap();

//         // stream.send_response(response).await.unwrap();
//         // stream.send_data(Bytes::from(res.body.to_owned()).clone()).await.unwrap();
//         // stream.finish().await.unwrap();

//         Ok(())
//     }

// }


