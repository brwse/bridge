use core::error::Error;
use std::collections::HashMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64_serde::base64_serde_type;
use bytes::BytesMut;
use cidr::{IpCidr, IpInet};
use eui48::MacAddress;
use geo_types::{LineString, Point, Rect};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use tokio_postgres::types::{
    FromSql, IsNull, Kind, ToSql, Type,
    private::{read_be_i32, read_value},
    to_sql_checked,
};
use uuid::Uuid;

base64_serde_type!(Base64Url, URL_SAFE_NO_PAD);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum Value {
    // Primitive types
    Array(Vec<Self>),
    Boolean(bool),
    Json(JsonValue),
    Null,
    Number(f64),
    String(String),
    Record(HashMap<String, Self>),
    Bytes(#[serde(with = "Base64Url")] Vec<u8>),

    // CIDR types
    IpCidr(IpCidr),
    IpInet(IpInet),
    MacAddress(MacAddress),

    // Geo types
    Line(LineString<f64>),
    Point(Point<f64>),
    Rect(Rect<f64>),

    // Time types
    Date(Date),
    Time(Time),
    Timestamp(PrimitiveDateTime),
    TimestampZoned(OffsetDateTime),

    // UUID types
    Uuid(Uuid),

    // Enum types
    Enum(String),
}

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match *self {
            Self::String(ref s) => s.to_sql(ty, out),
            Self::Array(ref params) => params.to_sql(ty, out),
            Self::Boolean(ref b) => b.to_sql(ty, out),
            Self::Json(ref value) => value.to_sql(ty, out),
            Self::Null => Ok(IsNull::Yes),
            Self::Number(ref n) => n.to_sql(ty, out),
            Self::Bytes(ref bytes) => bytes.to_sql(ty, out),
            Self::IpCidr(ref ip_cidr) => ip_cidr.to_sql(ty, out),
            Self::IpInet(ref ip_inet) => ip_inet.to_sql(ty, out),
            Self::MacAddress(ref mac_address) => mac_address.to_sql(ty, out),
            Self::Line(ref line_string) => line_string.to_sql(ty, out),
            Self::Point(ref point) => point.to_sql(ty, out),
            Self::Rect(ref rect) => rect.to_sql(ty, out),
            Self::Date(ref date) => date.to_sql(ty, out),
            Self::Time(ref time) => time.to_sql(ty, out),
            Self::Timestamp(ref primitive_date_time) => primitive_date_time.to_sql(ty, out),
            Self::TimestampZoned(ref offset_date_time) => offset_date_time.to_sql(ty, out),
            Self::Uuid(ref uuid) => uuid.to_sql(ty, out),
            Self::Enum(ref s) => {
                out.extend_from_slice(s.as_bytes());
                Ok(IsNull::No)
            }
            Self::Record(ref record) => to_composite(ty, out, record),
        }
    }

    fn accepts(_ty: &Type) -> bool {
        // We assume that all types are supported
        true
    }

    to_sql_checked!();
}

impl<'row> FromSql<'row> for Value {
    fn from_sql(ty: &Type, raw: &'row [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        if <String as FromSql>::accepts(ty) {
            return <String as FromSql>::from_sql(ty, raw).map(Value::String);
        }
        if <bool as FromSql>::accepts(ty) {
            return <bool as FromSql>::from_sql(ty, raw).map(Value::Boolean);
        }
        if <f64 as FromSql>::accepts(ty) {
            return <f64 as FromSql>::from_sql(ty, raw).map(Value::Number);
        }
        if <Vec<u8> as FromSql>::accepts(ty) {
            return <Vec<u8> as FromSql>::from_sql(ty, raw).map(Value::Bytes);
        }
        if <JsonValue as FromSql>::accepts(ty) {
            return <JsonValue as FromSql>::from_sql(ty, raw).map(Value::Json);
        }
        if <MacAddress as FromSql>::accepts(ty) {
            return <MacAddress as FromSql>::from_sql(ty, raw).map(Value::MacAddress);
        }
        if <LineString<f64> as FromSql>::accepts(ty) {
            return <LineString<f64> as FromSql>::from_sql(ty, raw).map(Value::Line);
        }
        if <Point<f64> as FromSql>::accepts(ty) {
            return <Point<f64> as FromSql>::from_sql(ty, raw).map(Value::Point);
        }
        if <Rect<f64> as FromSql>::accepts(ty) {
            return <Rect<f64> as FromSql>::from_sql(ty, raw).map(Value::Rect);
        }
        if <Date as FromSql>::accepts(ty) {
            return <Date as FromSql>::from_sql(ty, raw).map(Value::Date);
        }
        if <Time as FromSql>::accepts(ty) {
            return <Time as FromSql>::from_sql(ty, raw).map(Value::Time);
        }
        if <PrimitiveDateTime as FromSql>::accepts(ty) {
            return <PrimitiveDateTime as FromSql>::from_sql(ty, raw).map(Value::Timestamp);
        }
        if <OffsetDateTime as FromSql>::accepts(ty) {
            return <OffsetDateTime as FromSql>::from_sql(ty, raw).map(Value::TimestampZoned);
        }
        if <Uuid as FromSql>::accepts(ty) {
            return <Uuid as FromSql>::from_sql(ty, raw).map(Value::Uuid);
        }
        if <IpCidr as FromSql>::accepts(ty) {
            return <IpCidr as FromSql>::from_sql(ty, raw).map(Value::IpCidr);
        }
        if <IpInet as FromSql>::accepts(ty) {
            return <IpInet as FromSql>::from_sql(ty, raw).map(Value::IpInet);
        }
        if <Vec<Value> as FromSql>::accepts(ty) {
            return <Vec<Value> as FromSql>::from_sql(ty, raw).map(Value::Array);
        }
        match ty.kind() {
            Kind::Enum(_) => return <String as FromSql>::from_sql(ty, raw).map(Value::Enum),
            Kind::Composite(fields) => return from_composite(raw, fields),
            _ => {
                // Fallback to Null
            }
        };
        Ok(Value::Null)
    }

    fn accepts(_ty: &Type) -> bool {
        // We assume that all types are supported
        true
    }

    fn from_sql_null(_ty: &Type) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Self::Null)
    }
}

fn to_composite(
    ty: &Type,
    out: &mut BytesMut,
    record: &HashMap<String, Value>,
) -> Result<IsNull, Box<dyn Error + Send + Sync + 'static>> {
    let fields = match *ty.kind() {
        Kind::Composite(ref fields) => fields,
        _ => return Err(Box::new(std::io::Error::other("expected composite type"))),
    };
    out.extend_from_slice(&(fields.len() as i32).to_be_bytes());

    for field in fields {
        out.extend_from_slice(&field.type_().oid().to_be_bytes());

        let base = out.len();
        out.extend_from_slice(&[0; 4]);

        let r = record.get(field.name()).unwrap_or(&Value::Null).to_sql(field.type_(), out)?;

        let count = match r {
            IsNull::Yes => -1,
            IsNull::No => {
                let len = out.len() - base - 4;
                if len > i32::MAX as usize {
                    return Err(Box::new(std::io::Error::other("value too large to transmit")));
                }
                len as i32
            }
        };

        out[base..base + 4].copy_from_slice(&count.to_be_bytes());
    }

    Ok(IsNull::No)
}

fn from_composite<'row>(
    raw: &'row [u8],
    fields: &Vec<tokio_postgres::types::Field>,
) -> Result<Value, Box<dyn Error + Send + Sync + 'static>> {
    let mut buf = raw;
    let num_fields = read_be_i32(&mut buf)?;
    if num_fields as usize != fields.len() {
        return Err(Box::new(std::io::Error::other(format!(
            "invalid field count: {} vs {}",
            num_fields,
            fields.len(),
        ))));
    }
    let mut record = HashMap::new();
    for field in fields {
        let oid = read_be_i32(&mut buf)? as u32;
        if oid != field.type_().oid() {
            return Err(Box::new(std::io::Error::other("unexpected OID")));
        }

        record.insert(field.name().to_owned(), read_value(field.type_(), &mut buf)?);
    }
    Ok(Value::Record(record))
}
