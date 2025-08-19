use std::io::{Read, Result, Write};

use crate::flyer::{
    request::Request,
    response::Response,
    response::{new_response, parse},
    HTTP
};

// Try function ->
pub fn handle<'a, T: Write + Read>(server: &'a mut HTTP, mut socket: T, req: &mut Request) -> Result<()> {
    match server.router.match_web_routes(req) {
        Some(route) => {
            let res = &mut new_response();

            (route.route)(req, res);
            
            let _ = socket.write( parse(res)?.as_bytes())?;
            
            Ok(())
        },
        None => {
            match server.router.not_found_callback {
                Some(route) => {
                    let mut res: Response = new_response();

                    route(req, &mut res);

                    let _ = socket.write_all(parse(&mut res)?.as_bytes());

                    Ok(())
                },
                None => {
                    let mut res: Response = new_response();

                    res.status_code(404);

                    let _ = socket.write_all(parse(&mut res)?.as_bytes());

                    Ok(())
                },
            }
        },
    }
}