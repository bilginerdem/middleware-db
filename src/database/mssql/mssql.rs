use std::io::{Error, ErrorKind};
 
use async_trait::async_trait;
use deadpool_tiberius::Pool;
use tiberius::{Client, Config, Query, SqlBrowser};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::{
  models::command::{CommandRequest, CommandResponse},
  settings, SyncError,
};
pub struct MsSql {}

async fn execute(req: CommandRequest) -> Result<CommandResponse, std::io::Error> {
  match get_connection().await {
    Ok(mut client) => {
      let mut query = Query::new(req.query);

      for p in req.parameters {
        query.bind(p.value);
      }

      let result = query.execute(&mut client).await;
      match result {
        Ok(r) => {
          let mut res = CommandResponse::default();
          res.row_affected = r.rows_affected().to_vec();
          Ok(res)
        }
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e.to_string())),
      }
    }
    Err(e) => {
      log::error!("MsSql connection error: {}", e);
      Err(std::io::Error::new(std::io::ErrorKind::Other, "MsSql connection"))
    }
  }
}

#[inline]
async fn get_connection() -> Result<Client<Compat<TcpStream>>, Box<SyncError>> {
  let conn_str = settings::get_string("odbc.connection", None);
  if let Ok(mut db_config) = Config::from_ado_string(conn_str.as_str()) {
    db_config.trust_cert();
    match tokio::net::TcpStream::connect_named(&db_config).await {
      Ok(tcp) => match tiberius::Client::connect(db_config, tcp.compat_write()).await {
        Ok(client) => Ok(client),
        Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, e))),
      },
      Err(e) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, e))),
    } 
  } else {
    Err(Box::new(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "odbc.connection config error",
    )))
  }
} 

 