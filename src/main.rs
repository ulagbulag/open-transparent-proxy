use std::net::SocketAddr;

use actix_web::{
    web::{self, BytesMut},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use anyhow::{anyhow, Result};
use ark_core::{
    env::{self, infer},
    logger,
};
use futures::StreamExt;
use log::{info, warn};
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
    path: web::Path<String>,
) -> impl Responder {
    let scheme = req.connection_info().scheme().to_string();
    let host = req.connection_info().host().to_string();
    let path = path.into_inner();

    let Config {
        base_url,
        proxy_base_url,
        proxy_base_url_with_host,
        proxy_host,
    } = &**config;

    let mut builder = client.request(
        method.clone(),
        format!("{scheme}://{proxy_host}{proxy_base_url}{path}"),
    );

    for (key, value) in req.headers() {
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

        match match *key {
            header::HOST => patch_host(key, value, &host, &config.proxy_host).map(Some),
            header::ORIGIN => patch_host(key, value, &host, &config.proxy_host).map(Some),
            header::REFERER => patch_host(key, value, &host, &config.proxy_host).map(Some),
            ref key if key == header::HeaderName::from_static("x-forwarded-host") => Ok(None),
            _ => Ok(Some(value.clone())),
        } {
            Ok(Some(value)) => builder = builder.header(key, value),
            Ok(None) => {}
            Err(e) => return HttpResponse::Forbidden().body(e.to_string()),
        }

        if ![
            header::HOST,
            header::ORIGIN,
            header::REFERER,
            header::HeaderName::from_static("x-forwarded-host"),
        ]
        .contains(key)
        {
            builder = builder.header(key, value);
        }
    }

    // payload is a stream of Bytes objects
    let body = 'body: {
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
        Ok(buf.freeze())
    };
    builder = match body {
        Ok(body) => builder.body(body),
        Err(e) => return HttpResponse::Forbidden().body(e.to_string()),
    };

    match builder.send().await {
        Ok(res) => {
            let content_length = res.content_length();
            let status = res.status();
            info!("[{method}] {path:?} => {status}");

            let mut builder = HttpResponse::build(status);
            for (key, value) in res.headers() {
                builder.append_header((key, value));
            }
            if let Some(content_length) = content_length {
                builder.no_chunking(content_length);
            }

            builder.streaming(res.bytes_stream())
        }
        Err(e) => HttpResponse::Forbidden().body(format!("failed to find the url {path:?}: {e}")),
    }
}

struct Config {
    base_url: String,
    proxy_base_url: String,
    proxy_base_url_with_host: String,
    proxy_host: String,
}

impl Config {
    fn try_default() -> Result<Self> {
        let base_url = env::infer_string("BASE_URL").unwrap_or_else(|_| "/".into());
        let proxy_base_url =
            env::infer_string("PROXY_BASE_URL").unwrap_or_else(|_| base_url.clone());
        let proxy_host = env::infer_string("PROXY_HOST")?;

        let proxy_base_url_with_host = format!("{proxy_host}{proxy_base_url}");

        Ok(Self {
            base_url,
            proxy_base_url,
            proxy_base_url_with_host,
            proxy_host,
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

        let path = format!("/{base_url}{{path:.*}}", base_url = &config.base_url,);

        // Start web server
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::clone(&client))
                .app_data(web::Data::clone(&config))
                .route(&path, web::route().to(resolve))
        })
        .bind(addr)
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"))
        .run()
        .await
        .map_err(Into::into)
    }

    logger::init_once();
    try_main().await.expect("running a server")
}
