#[macro_use]
extern crate actix_web;
use actix_web::{middleware, web, App, HttpRequest, HttpServer, Result};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};

extern crate chrono;
use chrono::{DateTime, Utc};

static SERVER_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct AppState {
  server_id: usize,
  request_count: Cell<usize>,
  messages: Arc<Mutex<Vec<String>>>,
}

pub struct MessageApp {
  port: u16,
}

impl MessageApp {

  pub fn new(port: u16) -> Self {
    MessageApp { port }
  }

  pub fn run(&self) -> std::io::Result<()> {
    println!("Starting http server: 127.0.0.1:{}", self.port);
    let messages = Arc::new(Mutex::new(vec![]));
    HttpServer::new(move || {
      App::new()
        .data(AppState {
          server_id: SERVER_COUNTER.fetch_add(1, Ordering::SeqCst),
          request_count: Cell::new(0),
          messages: messages.clone()
        })
        .wrap(middleware::Logger::default())
        .service(index)
        .service(time)
        .service(clear)
        .service(
          web::resource("/send")
          .data(web::JsonConfig::default().limit(4096))
          .route(web::post().to(post_message))
        )
    })
    .bind(("127.0.0.1", self.port))?
    .workers(8)
    .run()
  }
}

#[derive(Deserialize)]
struct PostInput {
  message: String
}

#[derive(Serialize)]
struct PostResponse {
  server_id: usize,
  request_count: usize,
  message: String,
}

#[derive(Serialize)]
struct IndexResponse {
  server_id: usize,
  request_count: usize,
  messages: Vec<String>,
}

#[derive(Serialize)]
struct TimeResponse {
  rfc2822: String,
  timestamp: i64
}

#[get("/")]
fn index(state: web::Data<AppState>) -> Result<web::Json<IndexResponse>> {

  let request_count = state.request_count.get() + 1;
  state.request_count.set(request_count);
  let ms = state.messages.lock().unwrap();


  Ok(web::Json(IndexResponse {
    server_id: state.server_id,
    request_count,
    messages: ms.clone(),
  }))
}

fn post_message(msg: web::Json<PostInput>, state: web::Data<AppState>) -> Result<web::Json<PostResponse>> {
  let request_count = state.request_count.get() + 1;
  state.request_count.set(request_count);
  let mut ms = state.messages.lock().unwrap();
  ms.push(msg.message.clone());

  Ok(web::Json(PostResponse {
    server_id: state.server_id,
    request_count,
    message: msg.message.clone()
  }))
}

#[get("/now")]
fn time(_req: HttpRequest) -> Result<web::Json<TimeResponse>> {

  let now: DateTime<Utc> = Utc::now();
  Ok(web::Json(TimeResponse {
    rfc2822: now.to_rfc2822(),
    timestamp: now.timestamp_millis()
  }))
}
#[post("/clear")]
fn clear(state: web::Data<AppState>) -> Result<web::Json<IndexResponse>> {

  let request_count = state.request_count.get() + 1;
  state.request_count.set(request_count);

  let mut ms = state.messages.lock().unwrap();
  ms.clear();

  Ok(web::Json(IndexResponse {
    server_id: state.server_id,
    request_count,
    messages: ms.clone(),
  }))
}