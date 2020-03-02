#[macro_use]
extern crate actix_web;
use actix_web::{middleware, web, App, HttpRequest, HttpServer, Result};

use serde::Serialize;

extern crate chrono;
use chrono::{DateTime, Utc};

pub struct MessageApp {
  port: u16,
}

impl MessageApp {

  pub fn new(port: u16) -> Self {
    MessageApp { port }
  }

  pub fn run(&self) -> std::io::Result<()> {
    println!("Starting http server: 127.0.0.1:{}", self.port);
    HttpServer::new(move || {
      App::new()
        .wrap(middleware::Logger::default())
        .service(index)
        .service(time)
    })
    .bind(("127.0.0.1", self.port))?
    .workers(8)
    .run()
  }
}

#[derive(Serialize)]
struct IndexResponse {
  message: String,
}

#[get("/")]
fn index(req: HttpRequest) -> Result<web::Json<IndexResponse>> {
  let hello = req
    .headers()
    .get("hello")
    .and_then(|v| v.to_str().ok())
    .unwrap_or_else(|| "world");

  Ok(web::Json(IndexResponse {
    message: hello.to_owned(),
  }))
}

#[derive(Serialize)]
struct TimeResponse {
  rfc2822: String,
  timestamp: i64
}

#[get("/now")]
fn time(_req: HttpRequest) -> Result<web::Json<TimeResponse>> {

  let now: DateTime<Utc> = Utc::now();
  Ok(web::Json(TimeResponse {
    rfc2822: now.to_rfc2822(),
    timestamp: now.timestamp_millis()
  }))
}