use std::{
  fs::File,
  io::{self, BufReader, Read as _},
};

use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use middleware_db::{models::command::CommandRequest, settings};
use openssl::{
  pkey::{PKey, Private},
  ssl::{SslAcceptor, SslAcceptorBuilder, SslMethod},
};
use rustls::{pki_types::PrivateKeyDer, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

fn api_config(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::resource("/execute")
      .route(web::post().to(execute)) // Allow POST for command execution
      .route(web::get().to(healthcheck))
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

fn load_encrypted_private_key() -> PKey<Private> {
  let key_path = settings::get_string("ssl.key_path", None);
  let password = settings::get_string("ssl.key_password", None);

  let mut file = File::open(key_path).unwrap();
  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer).expect("Failed to read file");

  PKey::private_key_from_pem_passphrase(&buffer, password.as_bytes()).unwrap()
}

fn load_openssl_config() -> SslAcceptorBuilder {
  let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
  let cert_path = settings::get_string("ssl.cert_path", None);

  builder.set_private_key(&load_encrypted_private_key()).unwrap();
  builder.set_certificate_chain_file(cert_path).unwrap();
  builder
}

fn load_rustls_config() -> rustls::ServerConfig {
  rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

  // init server config builder with safe defaults
  let config = ServerConfig::builder().with_no_client_auth();

  let cert_path = settings::get_string("ssl.cert_path", None);
  let key_path = settings::get_string("ssl.key_path", None);
  // load TLS key/cert files
  let cert_file = &mut BufReader::new(File::open(cert_path).unwrap());
  let key_file = &mut BufReader::new(File::open(key_path).unwrap());

  // convert files to key/cert objects
  let cert_chain = certs(cert_file).collect::<Result<Vec<_>, _>>().unwrap();
  let mut keys = pkcs8_private_keys(key_file)
    .map(|key| key.map(PrivateKeyDer::Pkcs8))
    .collect::<Result<Vec<_>, _>>()
    .unwrap();

  // exit if no keys could be parsed
  if keys.is_empty() {
    eprintln!("Could not locate PKCS 8 private keys.");
    std::process::exit(1);
  }

  config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}

#[actix_web::main]
async fn main() -> io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=info");
  env_logger::init();
  dotenv().ok();

  let server_port: u16 = settings::get_int("server.port", Some(8443))
    .try_into()
    .expect("port must be in u16 range");
  let ssl_mode = settings::get_int("server.ssl_mode", Some(0));

  let http_server = HttpServer::new(|| {
    let cors = Cors::permissive(); // Adjust CORS as needed for production

    App::new()
      .wrap(middleware::NormalizePath::default())
      .wrap(cors)
      .wrap(middleware::Logger::default())
      .wrap(
        middleware::DefaultHeaders::new()
          .add(header::ContentType::json())
          .add(header::CacheControl(vec![header::CacheDirective::NoCache])),
      )
      .configure(app_config)
      .service(web::scope("/api").configure(api_config))
  })
  .workers(4);

  println!("Started https server: 127.0.0.1:{}", server_port);

  let address = format!("127.0.0.1:{}", server_port);

  if ssl_mode == 1 {
    let config = load_rustls_config();
    return http_server.bind_rustls_0_23(address, config)?.run().await;
  } else if ssl_mode == 2 {
    let config = load_openssl_config();
    return http_server.bind_openssl(address, config)?.run().await;
  } else {
    return http_server.bind(address)?.run().await;
  };
}

async fn execute(req: web::Json<CommandRequest>) -> impl actix_web::Responder {
  web::Json(req)
}

async fn healthcheck() -> impl actix_web::Responder {
  HttpResponse::Ok().body("OK")
}
