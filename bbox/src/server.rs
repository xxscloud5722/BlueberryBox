use std::convert::Infallible;
use std::ffi::OsStr;
use std::net::SocketAddr;
use hyper::{Body, HeaderMap, Method, Request, Response, Server, StatusCode};
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;
use log::{error, info};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use crate::{Args, core, setting};
use crate::core::Resource;

lazy_static! {
    static ref RESOURCE: RwLock<Option<Resource>> = RwLock::new(None);
    static ref EXCLUDE: Vec<&'static str> = vec!["/favicon.ico"];
}



pub async fn start(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let port = args.port;
    let config_path = args.config;
    let local_path = args.scan;
    let log_output = args.log;

    // 设置日志配置
    setting::setting_log(&log_output).await.unwrap();

    // 资源核心初始化
    let resource = Resource::from(&local_path, &config_path).await;
    resource.init_config().await;
    *RESOURCE.write().await = Some(resource);

    // 创建socket
    let socket_address = SocketAddr::from(([0, 0, 0, 0], port));

    info!("BlueberryBox started on port(s) {} (http) with scan path '{}'", port, local_path);

    // 启动服务
    if let Err(e) = Server::bind(&socket_address).serve(make_service_fn(|_| async {
        Ok::<_, Infallible>(service_fn(request_handle))
    })).await {
        error!("BlueberryBox Server Error: {}", e);
    }
    Ok(())
}

async fn request_handle(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    // 过滤非Get 请求方式
    if request.method() != &Method::GET {
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    // 如果资源没有准备就绪
    let resource = &*RESOURCE.read().await;
    if resource.is_none() {
        return Ok(response);
    }
    let resource = resource.as_ref().unwrap();

    // 如果静态资源不存在
    match resource.is_static_file(request.uri().path()).await {
        None => {}
        Some(value) => {
            let file = File::open(&value).await.unwrap();
            *response.body_mut() = Body::wrap_stream(tokio_util::io::ReaderStream::new(file));
            let extension = match &value.as_path().extension().and_then(OsStr::to_str) {
                None => "",
                Some(value) => value
            };
            response_content_type(response.headers_mut(), extension);
            return Ok(response);
        }
    }


    // 如果首页文件存在
    if !EXCLUDE.contains(&request.uri().path()) && resource.is_index().await {
        *response.body_mut() = Body::from(resource.parse_html_by_config(request.uri().path()).await);
        response_content_type(response.headers_mut(), "html");
        return Ok(response);
    }


    *response.status_mut() = StatusCode::NOT_FOUND;
    return Ok(response);
}

async fn response_content_type(header: &mut HeaderMap<HeaderValue>, format: &str) {
    let format = format.to_lowercase();
    if format == "txt" {
        (*header).append("Content-Type", ("text/plan; charset=UTF-8").parse().unwrap());
    }
    if format == "xml" || format == "json" {
        (*header).append("Content-Type", format!("text/{}; charset=UTF-8", format).parse().unwrap());
    }
    if format == "pdf" {
        (*header).append("Content-Type", format!("application/{}; charset=UTF-8", format).parse().unwrap());
    }
    if format == "xls" || format == "xlsx" {
        (*header).append("Content-Type", ("application/octet-stream; charset=UTF-8").parse().unwrap());
    }
    if format == "js" || format == "css" || format == "html" {
        (*header).append("Content-Type", format!("text/{}", format).parse().unwrap());
    }
    if format == "png" || format == "jpg" || format == "jpeg" || format == "gif" {
        (*header).append("Content-Type", format!("image/{}", format).parse().unwrap());
    }
    if format == "mp3" || format == "mpeg" || format == "ogg" || format == "wav" {
        (*header).append("Content-Type", format!("audio/{}", format).parse().unwrap());
    }
    if format == "mp4" || format == "webm" || format == "ogg" {
        (*header).append("Content-Type", format!("video/{}", format).parse().unwrap());
    }
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
