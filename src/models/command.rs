use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use config::Map;
use tiberius::IntoSql;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum DbValue {
  Binary(Vec<u8>),
  Byte(u8),
  Boolean(bool),
  Currency(f64),
  Date(chrono::NaiveDate),
  DateTime(chrono::NaiveDateTime),
  Decimal(f32),
  Double(f32),
  Guid(String),
  Int16(u16),
  Int32(u32),
  Int64(u64),
  SByte(i8),
  Single(f32),
  String(String),
  Time(chrono::NaiveTime),
  UInt16(u16),
  UInt32(u32),
  UInt64(u64),
  Xml(String),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandParameter {
  pub name: String,
  pub type_name: String,
  pub direction: String,
  pub value: DbValue,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CommandRequest {
  pub query: String,
  pub parameters: Vec<CommandParameter>,
}

impl<'a> IntoSql<'a> for DbValue {
  fn into_sql(self) -> tiberius::ColumnData<'a> {
    match self {
      DbValue::Binary(v) => v.into_sql(),
      DbValue::Byte(v) => v.into_sql(),
      DbValue::SByte(v) => (v as u8).into_sql(),
      DbValue::Boolean(v) => v.into_sql(),
      DbValue::Currency(v) => v.into_sql(),
      DbValue::Date(v) => v.into_sql(),
      DbValue::DateTime(v) => v.into_sql(),
      DbValue::Decimal(v) => v.into_sql(),
      DbValue::Double(v) => v.into_sql(),
      DbValue::Guid(v) => tiberius::ColumnData::Guid(Some(v.parse().unwrap_or_default())),
      DbValue::Int16(v) | DbValue::UInt16(v) => tiberius::ColumnData::I16(Some(v as i16)),
      DbValue::Int32(v) | DbValue::UInt32(v) => tiberius::ColumnData::I32(Some(v as i32)),
      DbValue::Int64(v) | DbValue::UInt64(v) => tiberius::ColumnData::I64(Some(v as i64)),
      DbValue::Time(v) => v.into_sql(),
      DbValue::Single(v) => v.into_sql(),
      DbValue::String(v) => v.into_sql(),
      DbValue::Xml(v) => v.into_sql(),
    }
  }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct CommandResponse {
  pub success: bool,
  pub message: String,
  pub record_sets: Vec<Vec<Map<String, DbValue>>>,
  pub row_affected: Vec<u64>,
  pub output: Vec<DbValue>,
}

impl Responder for CommandResponse {
  type Body = BoxBody;

  fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
    let body = serde_json::to_string(&self).unwrap();
    HttpResponse::Ok().content_type(ContentType::json()).body(body)
  }
}
