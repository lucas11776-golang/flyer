use std::mem::take;

use crate::router::{
    RouteNotFoundCallback,
    RouterNodes,
    WebRoutes,
    WsRoutes
};

pub fn resolve_router_nodes<'a>(nodes: &'a mut RouterNodes) -> (WebRoutes, WsRoutes, RouteNotFoundCallback) {
    let mut web = WebRoutes::new();
    let mut ws = WsRoutes::new();
    let mut not_found = RouteNotFoundCallback::None;


    for node in &mut *nodes {
        if let Some(group) = node.group {
            group(node);
        }
        

        web.extend(take(&mut node.web));
        ws.extend(take(&mut node.ws));
        

        if node.not_found.is_none() {
            not_found = take(&mut node.not_found);                
        }

        let (temp_web, temp_ws, temp_not_found) = resolve_router_nodes(&mut node.nodes);

        web.extend(temp_web);
        ws.extend(temp_ws);


        if temp_not_found.is_none() {
            not_found = temp_not_found;                
        }
    }

    return (web, ws, not_found);
}