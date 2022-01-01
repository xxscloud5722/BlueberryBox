use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc::SendError;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use log::{error, info};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use visdom::Vis;
use crate::{core, global, setting};


pub async fn start(args: HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let port = match args.get("p") {
        None => 3000,
        Some(v) => v.parse::<u16>().unwrap()
    };
    let config_path = match args.get("c") {
        None => {
            let mut root = env::current_dir().unwrap();
            root.push("config.json");
            root.as_os_str().to_str().unwrap().to_string()
        }
        Some(v) => v.to_string()
    };
    let local_path = match args.get("path") {
        None => {
            let mut root = env::current_dir().unwrap();
            root.push("static");
            root.as_os_str().to_str().unwrap().to_string()
        }
        Some(v) => v.to_string()
    };
    let log_output = match args.get("log") {
        None => env::current_dir().unwrap().as_os_str().to_str().unwrap().to_string(),
        Some(v) => v.to_string()
    };

    // global
    global::set_local_path(&local_path);
    let resource = core::Resource::from(&local_path, &config_path);
    resource.handel().await;
    global::set_resource(resource);

    // load log service
    setting::setting_log(&log_output).unwrap();

    // create socket
    let socket_address = SocketAddr::from(([0, 0, 0, 0], port));

    info!("BlueberryBox started on port(s) {} (http) with scan path '{}'", port, local_path);

    // start server
    if let Err(e) = Server::bind(&socket_address).serve(make_service_fn(|_| async {
        Ok::<_, Infallible>(service_fn(request_handle))
    })).await {
        error!("BlueberryBox Server Error: {}", e);
    }
    Ok(())
}

async fn request_handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    // Non get Filter
    if request.method() != &Method::GET {
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let resource = global::get_resource();

    // file is exist
    match resource.check_url(request.uri().path()).await {
        None => {}
        Some(value) => {
            let file = File::open(value).await.unwrap();
            *response.body_mut() = Body::wrap_stream(tokio_util::io::ReaderStream::new(file));
            return Ok(response);
        }
    }


    // index.html
    if resource.is_index().await {
        *response.body_mut() = Body::from(resource.parse_html_by_config(request.uri().path()).await);
        return Ok(response);
    }


    *response.status_mut() = StatusCode::NOT_FOUND;
    return Ok(response);
}

// if std::path::Path::new(value.as_os_str()).exists() {
//
// }

// async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
//     //Ok(Response::new("Hello, World".into()))
//     let mut file = tokio::fs::File::open("/Users/xxscloud/code/github/BlueberryBox/bbox/index.html").await.unwrap();
//     // let mut file = std::io::BufReader::new(file);
//     // let mut file = file.buffer().clone();
//     // let mut buffer = Vec::new();
//     // file.read_to_end(&mut buffer).unwrap();
//     //Ok(Response::new(_req.into_body().into()))
//     //tokio::codec::FramedRead::new(file, BytesCodec::new())
//     let html = r##"
//     <!DOCTYPE html>
//     <html>
//       <head>
//         <meta charset="utf-8" />
//       </head>
//       <body>
//         <nav id="header">
//           <ul>
//             <li>Hello,</li>
//             <li>Vis</li>
//             <li>Dom</li>
//           </ul>
//         </nav>
//       </body>
//     </html>
//   "##;
//     let root = Vis::load(html).unwrap();
//     let node = r##"<title>Rust visdom_百度搜索</title> <meta http-equiv="X-UA-Compatible" content="IE=edge,chrome=1">"##;
//     root.find("head").append(&mut Vis::load(node).unwrap());
//     // Ok(Response::new(Body::wrap_stream(tokio_util::io::ReaderStream::new(file)).into()))
//     // 监听文件变化
//     Ok(Response::new(root.html().into()))
// }
//
// async fn file(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
//     let full_body = hyper::body::to_bytes(_req.into_body()).await.unwrap();
//     Ok(Response::new(full_body.into()))
// }
