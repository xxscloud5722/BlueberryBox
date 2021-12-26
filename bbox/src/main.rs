use log::info;

mod banner;
mod setting;
mod server;
mod resource;

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

fn main() {
    banner::banner();
    setting::init();
    server();
}

#[tokio::main]
async fn server() {
    let resource = resource::Resource::from();
    info!("{}",resource.check_url("123"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("BlueberryBox started successfully !");
    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}
