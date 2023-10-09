use std::io;

use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use middleware_db::models::command::{CommandRequest, CommandResponse};
//use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

fn api_config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("/execute")
      .route(web::post().to(execute))
      .route(web::trace().to(HttpResponse::MethodNotAllowed))
      .route(web::head().to(HttpResponse::MethodNotAllowed)),
  );
}

fn app_config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("/app")
      .route(web::get().to(|| async { HttpResponse::Ok().body("OK") }))
      .route(web::trace().to(HttpResponse::MethodNotAllowed))
      .route(web::head().to(HttpResponse::MethodNotAllowed)),
  );
}

#[actix_web::main]
async fn main() -> io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=info");

  println!("Started http server: 127.0.0.1:8080");

  //let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
  // builder.set_private_key_file("./key.pem", SslFiletype::PEM)?;
  // builder.set_certificate_chain_file("./cert.pem")?;

  HttpServer::new(|| {
    App::new()
      .wrap(middleware::NormalizePath::default())
      .configure(app_config)
      .service(web::scope("/api").configure(api_config))
  })
  .workers(4)
  //.bind_openssl("127.0.0.1:8443", builder)?
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}

async fn execute(req: web::Json<CommandRequest>) -> Result<CommandResponse> {
  let obj = CommandResponse::default();
  Ok(obj)
}
