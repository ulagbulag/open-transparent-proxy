use std::net::SocketAddr;

use actix_web::{
    web::{self, BytesMut},
    App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Responder,
};
use anyhow::{anyhow, Result};
use ark_core::{
    env::{self, infer},
    logger,
};
use futures::StreamExt;
use log::{info, warn};
use regex::Regex;
use reqwest::{
    header::{self, HeaderName, HeaderValue},
    Client, ClientBuilder, Method,
};

async fn resolve(
    client: web::Data<Client>,
    config: web::Data<Config>,
    req: HttpRequest,
    method: Method,
    mut payload: web::Payload,
) -> impl Responder {
    fn patch_host(
        key: &HeaderName,
        value: &HeaderValue,
        src: &str,
        target: &str,
    ) -> Result<HeaderValue> {
        let error = || anyhow!("invalid header: {key}");

        value
            .to_str()
            .map_err(|_| error())
            .map(|value| value.replace(src, target))
            .and_then(|value| HeaderValue::from_str(&value).map_err(|_| error()))
    }

    // load proxy config
    let Config {
        base_url,
        proxy_base_url,
        proxy_base_url_with_host,
        proxy_host,
        proxy_scheme,
    } = &**config;

    // get basic request information
    let scheme = req.connection_info().scheme().to_string();
    let host = req.connection_info().host().to_string();
    let peer_addr = req
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "unknown".into());
    let query = match req.query_string() {
        "" => Default::default(),
        query => format!("?{query}"),
    };

    // parse path
    let path = req.path();
    let path = if path.starts_with(base_url) {
        &path[base_url.len()..]
    } else {
        path
    };
    let proxy_path = format!("{proxy_base_url}{path}{query}");
    let proxy_url = format!("{proxy_scheme}://{proxy_host}{proxy_path}");

    // define a request
    let mut builder = client.request(method.clone(), &proxy_url);
    for (key, value) in req.headers() {
        match match *key {
            header::ACCEPT_ENCODING | header::CONNECTION => Ok(None),
            header::HOST | header::ORIGIN | header::REFERER => {
                patch_host(key, value, &host, &config.proxy_host).map(Some)
            }
            ref key if key == header::HeaderName::from_static("x-forwarded-host") => Ok(None),
            _ => Ok(Some(value.clone())),
        } {
            Ok(Some(value)) => builder = builder.header(key, value),
            Ok(None) => {}
            Err(e) => return HttpResponse::Forbidden().body(e.to_string()),
        }
    }

    // load a payload, which is a stream of Bytes objects
    let body = 'body: {
        match method {
            Method::PATCH | Method::POST | Method::PUT => {
                let mut buf = BytesMut::new();
                while let Some(chunk) = payload.next().await {
                    const MAX_SIZE: usize = 262_144; // max payload size is 256k

                    match chunk {
                        // limit max size of in-memory payload
                        Ok(chunk) if (buf.len() + chunk.len()) <= MAX_SIZE => {
                            buf.extend_from_slice(&chunk);
                        }
                        Ok(_) => {
                            break 'body Err("Overflowed");
                        }
                        Err(e) => {
                            warn!("failed to get bytes: {e}");
                            break 'body Err("Err");
                        }
                    }
                }
                Ok(Some(buf.freeze()))
            }
            _ => Ok(None),
        }
    };
    builder = match body {
        Ok(Some(body)) => builder.body(body),
        Ok(None) => builder,
        Err(e) => return HttpResponse::Forbidden().body(e.to_string()),
    };

    // call a proxy request
    let (res, status) = match builder.send().await {
        Ok(res) => {
            let status = res.status();
            info!("[{method}] {peer_addr} => /{path}{query} => {status}");
            (res, status)
        }
        Err(e) => {
            return HttpResponse::Forbidden().body(format!("failed to find the url {path:?}: {e}"))
        }
    };

    // define a response builder
    let mut builder = HttpResponse::build(status);
    for (key, value) in res.headers() {
        match *key {
            header::CONTENT_ENCODING => {}
            header::CONTENT_LENGTH => {}
            header::CONTENT_SECURITY_POLICY => {}
            _ => match patch_host(key, value, &config.proxy_host, &host) {
                Ok(value) => {
                    builder.append_header((key, value));
                }
                Err(e) => return HttpResponse::Forbidden().body(e.to_string()),
            },
        }
    }

    fn respond_pass_through(
        mut builder: HttpResponseBuilder,
        res: ::reqwest::Response,
    ) -> HttpResponse {
        if let Some(content_length) = res.content_length() {
            builder.no_chunking(content_length);
        }
        builder.streaming(res.bytes_stream())
    }

    // send a response
    match res.headers().get(header::CONTENT_TYPE) {
        Some(content_type) => match content_type.to_str() {
            Ok(content_type) => match content_type.parse::<::mime::Mime>() {
                Ok(mime) => match mime.subtype() {
                    ::mime::HTML => match res.text().await {
                        Ok(body) => {
                            let body = {
                                let re = Regex::new(r#"(href=")/"#).unwrap();
                                re.replace_all(&body, format!(r#"$0"#))
                            };
                            let body = {
                                let re = Regex::new(r#"(src=")/"#).unwrap();
                                re.replace_all(&body, format!(r#"$0"#))
                            };
                            let body = {
                                let re = Regex::new(r#"(url=")/"#).unwrap();
                                re.replace_all(&body, format!(r#"$0"#))
                            };
                            let body = {
                                let re = Regex::new(r#"<head[ \.\_\-\=A-Za-z0-9'"]*>"#).unwrap();
                                re.replace_all(&body, format!(r#"$0<base href="{base_url}">"#))
                            };
                            builder.body(body.to_string())
                        }
                        Err(e) => HttpResponse::Forbidden()
                            .body(format!("failed to parse the response body as string: {e}")),
                    },
                    _ => respond_pass_through(builder, res),
                },
                Err(e) => HttpResponse::Forbidden()
                    .body(format!("failed to parse the response content type: {e}")),
            },
            Err(e) => HttpResponse::Forbidden().body(format!(
                "failed to parse the response content type as string: {e}"
            )),
        },
        None => respond_pass_through(builder, res),
    }
}

struct Config {
    base_url: String,
    proxy_base_url: String,
    proxy_base_url_with_host: String,
    proxy_host: String,
    proxy_scheme: String,
}

impl Config {
    fn try_default() -> Result<Self> {
        let base_url = env::infer_string("BASE_URL").unwrap_or_else(|_| "/".into());
        let proxy_base_url =
            env::infer_string("PROXY_BASE_URL").unwrap_or_else(|_| base_url.clone());
        let proxy_host = env::infer_string("PROXY_HOST")?;
        let proxy_scheme = env::infer_string("PROXY_SCHEME").unwrap_or_else(|_| "https".into());

        let proxy_base_url_with_host = format!("{proxy_host}{proxy_base_url}");

        Ok(Self {
            base_url,
            proxy_base_url,
            proxy_base_url_with_host,
            proxy_host,
            proxy_scheme,
        })
    }
}

#[actix_web::main]
async fn main() {
    async fn try_main() -> Result<()> {
        // Initialize kubernetes client
        let addr =
            infer::<_, SocketAddr>("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:80".parse().unwrap());

        // Initialize client
        let client = web::Data::new({
            let builder = ClientBuilder::new();
            builder
                .build()
                .map_err(|e| anyhow!("failed to init reqwest client: {e}"))?
        });
        let config = web::Data::new(
            Config::try_default().map_err(|e| anyhow!("failed to parse config: {e}"))?,
        );

        let path = format!("{base_url}{{path:.*}}", base_url = &config.base_url,);

        // Start web server
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::clone(&client))
                .app_data(web::Data::clone(&config))
                .route(&path, web::route().to(resolve))
        })
        .bind(addr)
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"))
        .shutdown_timeout(20)
        .run()
        .await
        .map_err(Into::into)
    }

    logger::init_once();
    try_main().await.expect("running a server")
}
