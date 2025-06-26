use core::error::Error;

use base64::prelude::BASE64_STANDARD;
use base64_serde::base64_serde_type;
use bytes::BytesMut;
use cidr::{IpCidr, IpInet};
use eui48::MacAddress;
use geo_types::{LineString, Point, Rect};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use time::{Date, OffsetDateTime, Time};
use tokio_postgres::types::{
    FromSql, IsNull, Kind, ToSql, Type,
    private::{read_be_i32, read_value},
    to_sql_checked,
};
use uuid::Uuid;

use crate::schema::remove_excess;

base64_serde_type!(Base64Url, BASE64_STANDARD);

mod serde_serde {
    pub mod str {
        use core::{fmt::Display, str::FromStr};

        use serde::{Deserialize as _, Deserializer, Serialize, Serializer};

        pub fn serialize<S: Serializer, T: ToString>(
            value: &T,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            value.to_string().serialize(serializer)
        }

        pub fn deserialize<'a, D: Deserializer<'a>, T: FromStr<Err = E>, E: Display>(
            deserializer: D,
        ) -> Result<T, D::Error> {
            let s = String::deserialize(deserializer)?;
            T::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    pub mod mac_address {
        use core::str::FromStr;

        use eui48::MacAddress;
        use serde::{Deserialize as _, Deserializer, Serialize, Serializer};

        pub fn serialize<S: Serializer>(
            value: &MacAddress,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            value.to_hex_string().serialize(serializer)
        }

        pub fn deserialize<'a, D: Deserializer<'a>>(
            deserializer: D,
        ) -> Result<MacAddress, D::Error> {
            let s = String::deserialize(deserializer)?;
            MacAddress::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    pub mod date {
        use serde::{Deserialize as _, Deserializer};
        use time::{Date, format_description::well_known::Rfc3339};

        pub fn deserialize<'a, D: Deserializer<'a>>(deserializer: D) -> Result<Date, D::Error> {
            let s = String::deserialize(deserializer)?;
            Date::parse(&s, &Rfc3339).map_err(serde::de::Error::custom)
        }
    }

    pub mod time {
        use serde::{Deserialize as _, Deserializer};
        use time::{Time, format_description::well_known::Rfc3339};

        pub fn deserialize<'a, D: Deserializer<'a>>(deserializer: D) -> Result<Time, D::Error> {
            let s = String::deserialize(deserializer)?;
            Time::parse(&s, &Rfc3339).map_err(serde::de::Error::custom)
        }
    }

    pub mod primitive_date_time {
        use serde::{Deserialize as _, Deserializer};
        use time::{PrimitiveDateTime, format_description::well_known::Rfc3339};

        pub fn deserialize<'a, D: Deserializer<'a>>(
            deserializer: D,
        ) -> Result<PrimitiveDateTime, D::Error> {
            let s = String::deserialize(deserializer)?;
            PrimitiveDateTime::parse(&s, &Rfc3339).map_err(serde::de::Error::custom)
        }
    }
}

pub fn value_schema(_generator: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "description": "A PostgreSQL value",
        "type": ["object", "array", "string", "number", "boolean", "null"]
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(schema_with = "value_schema", transform = remove_excess)]
pub enum Value {
    Boolean(bool),
    Null,
    Number(f64),

    Uuid(Uuid),
    Timestamp(#[serde(with = "time::serde::rfc3339")] OffsetDateTime),
    Date(
        #[serde(
            serialize_with = "serde_serde::str::serialize",
            deserialize_with = "serde_serde::date::deserialize"
        )]
        Date,
    ),
    Time(
        #[serde(
            serialize_with = "serde_serde::str::serialize",
            deserialize_with = "serde_serde::time::deserialize"
        )]
        Time,
    ),
    IpCidr(#[serde(with = "serde_serde::str")] IpCidr),
    IpInet(#[serde(with = "serde_serde::str")] IpInet),
    MacAddress(#[serde(with = "serde_serde::mac_address")] MacAddress),
    String(String),

    // Geo types
    Line(#[schemars(schema_with = "schema::line_string")] LineString<f64>),
    Point(#[schemars(schema_with = "schema::point")] Point<f64>),
    Rect(#[schemars(schema_with = "schema::rect")] Rect<f64>),

    Array(Vec<Self>),
    Json(JsonValue),
}

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        if let Kind::Domain(domain) = ty.kind() {
            return self.to_sql(domain, out);
        }
        match *self {
            Self::Array(ref params) => params.to_sql(ty, out),
            Self::Boolean(ref b) => b.to_sql(ty, out),
            Self::Null => Ok(IsNull::Yes),
            Self::Number(ref n) => n.to_sql(ty, out),
            Self::Uuid(ref uuid) => {
                if <String as ToSql>::accepts(ty) {
                    uuid.to_string().to_sql(ty, out)
                } else {
                    uuid.to_sql(ty, out)
                }
            }
            Self::Date(ref date) => {
                if <String as ToSql>::accepts(ty) {
                    date.to_string().to_sql(ty, out)
                } else {
                    date.to_sql(ty, out)
                }
            }
            Self::Time(ref time) => {
                if <String as ToSql>::accepts(ty) {
                    time.to_string().to_sql(ty, out)
                } else {
                    time.to_sql(ty, out)
                }
            }
            Self::Timestamp(ref primitive_date_time) => {
                if <String as ToSql>::accepts(ty) {
                    primitive_date_time.to_string().to_sql(ty, out)
                } else {
                    primitive_date_time.to_sql(ty, out)
                }
            }
            Self::IpCidr(ref ip_cidr) => {
                if <String as ToSql>::accepts(ty) {
                    ip_cidr.to_string().to_sql(ty, out)
                } else {
                    ip_cidr.to_sql(ty, out)
                }
            }
            Self::IpInet(ref ip_inet) => {
                if <String as ToSql>::accepts(ty) {
                    ip_inet.to_string().to_sql(ty, out)
                } else {
                    ip_inet.to_sql(ty, out)
                }
            }
            Self::MacAddress(ref mac_address) => {
                if <String as ToSql>::accepts(ty) {
                    mac_address.to_string(eui48::MacAddressFormat::HexString).to_sql(ty, out)
                } else {
                    mac_address.to_sql(ty, out)
                }
            }
            Self::String(ref s) => {
                if matches!(ty.kind(), Kind::Enum(_)) {
                    out.extend_from_slice(s.as_bytes());
                    Ok(IsNull::No)
                } else {
                    s.to_sql(ty, out)
                }
            }
            Self::Line(ref line) => line.to_sql(ty, out),
            Self::Point(ref point) => point.to_sql(ty, out),
            Self::Rect(ref rect) => rect.to_sql(ty, out),
            Self::Json(ref json) => json.to_sql(ty, out),
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
        if <Vec<Self> as FromSql>::accepts(ty) {
            return <Vec<Self> as FromSql>::from_sql(ty, raw).map(Value::Array);
        }
        if <bool as FromSql>::accepts(ty) {
            return <bool as FromSql>::from_sql(ty, raw).map(Value::Boolean);
        }
        if <f64 as FromSql>::accepts(ty) {
            return <f64 as FromSql>::from_sql(ty, raw).map(Value::Number);
        }
        if <Uuid as FromSql>::accepts(ty) {
            return <Uuid as FromSql>::from_sql(ty, raw).map(Self::Uuid);
        }
        if <OffsetDateTime as FromSql>::accepts(ty) {
            return <OffsetDateTime as FromSql>::from_sql(ty, raw).map(Self::Timestamp);
        }
        if <Date as FromSql>::accepts(ty) {
            return <Date as FromSql>::from_sql(ty, raw).map(Self::Date);
        }
        if <Time as FromSql>::accepts(ty) {
            return <Time as FromSql>::from_sql(ty, raw).map(Self::Time);
        }
        if <IpCidr as FromSql>::accepts(ty) {
            return <IpCidr as FromSql>::from_sql(ty, raw).map(Self::IpCidr);
        }
        if <IpInet as FromSql>::accepts(ty) {
            return <IpInet as FromSql>::from_sql(ty, raw).map(Self::IpInet);
        }
        if <MacAddress as FromSql>::accepts(ty) {
            return <MacAddress as FromSql>::from_sql(ty, raw).map(Self::MacAddress);
        }
        if <String as FromSql>::accepts(ty) {
            return <String as FromSql>::from_sql(ty, raw).map(Self::String);
        }
        if <LineString<f64> as FromSql>::accepts(ty) {
            return <LineString<f64> as FromSql>::from_sql(ty, raw).map(Self::Line);
        }
        if <Point<f64> as FromSql>::accepts(ty) {
            return <Point<f64> as FromSql>::from_sql(ty, raw).map(Self::Point);
        }
        if <Rect<f64> as FromSql>::accepts(ty) {
            return <Rect<f64> as FromSql>::from_sql(ty, raw).map(Self::Rect);
        }
        if <JsonValue as FromSql>::accepts(ty) {
            return <JsonValue as FromSql>::from_sql(ty, raw).map(Self::Json);
        }
        match ty.kind() {
            Kind::Enum(_) => return <String as FromSql>::from_sql(ty, raw).map(Self::String),
            Kind::Composite(fields) => return from_composite(raw, fields),
            Kind::Domain(domain) => return Self::from_sql(domain, raw),
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

fn to_json_or_composite(
    ty: &Type,
    out: &mut BytesMut,
    record: &JsonValue,
) -> Result<IsNull, Box<dyn Error + Send + Sync + 'static>> {
    if <JsonValue as ToSql>::accepts(ty) {
        return record.to_sql(ty, out);
    }
    match *ty.kind() {
        Kind::Composite(ref fields) => {
            out.extend_from_slice(&(fields.len() as i32).to_be_bytes());

            for field in fields {
                out.extend_from_slice(&field.type_().oid().to_be_bytes());

                let base = out.len();
                out.extend_from_slice(&[0; 4]);

                let r = record
                    .get(field.name())
                    .unwrap_or(&JsonValue::Null)
                    .to_sql(field.type_(), out)?;

                let count = match r {
                    IsNull::Yes => -1,
                    IsNull::No => {
                        let len = out.len() - base - 4;
                        if len > i32::MAX as usize {
                            return Err(Box::new(std::io::Error::other(
                                "value too large to transmit",
                            )));
                        }
                        len as i32
                    }
                };

                out[base..base + 4].copy_from_slice(&count.to_be_bytes());
            }

            Ok(IsNull::No)
        }
        _ => Err(Box::new(std::io::Error::other("expected composite type"))),
    }
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
    let mut record = serde_json::Map::new();
    for field in fields {
        let oid = read_be_i32(&mut buf)? as u32;
        if oid != field.type_().oid() {
            return Err(Box::new(std::io::Error::other("unexpected OID")));
        }

        record.insert(field.name().to_owned(), read_value(field.type_(), &mut buf)?);
    }
    Ok(Value::Json(JsonValue::Object(record)))
}

#[cfg(test)]
mod tests {
    use core::net::{IpAddr, Ipv4Addr};

    use insta::assert_json_snapshot;
    use schemars::schema_for;
    use serde_json::json;
    use time::Month;

    use super::*;

    fn get_value() -> Vec<Value> {
        vec![
            Value::String("test".to_string()),
            Value::Boolean(true),
            Value::Number(1.0),
            Value::Json(JsonValue::Object(serde_json::Map::from_iter(vec![(
                "test".to_string(),
                JsonValue::String("test".to_string()),
            )]))),
            Value::Null,
            Value::Array(vec![Value::String("test".to_string())]),
            Value::IpCidr(IpCidr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)), 24).unwrap()),
            Value::IpInet(IpInet::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0)), 24).unwrap()),
            Value::MacAddress(MacAddress::nil()),
            Value::Line(LineString::from(vec![(0.0, 0.0), (1.0, 1.0)])),
            Value::Point(Point::new(0.0, 0.0)),
            Value::Rect(Rect::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0))),
            Value::Date(Date::from_calendar_date(2021, Month::January, 1).unwrap()),
            Value::Time(Time::from_hms(12, 34, 56).unwrap()),
            Value::Timestamp(OffsetDateTime::new_utc(
                Date::from_calendar_date(2021, Month::January, 1).unwrap(),
                Time::from_hms(12, 34, 56).unwrap(),
            )),
            Value::Uuid(Uuid::from_bytes([0; 16])),
        ]
    }

    #[test]
    fn test_value() {
        let value = get_value();

        assert_json_snapshot!(value, @r###"
        [
          "test",
          true,
          1.0,
          {
            "test": "test"
          },
          null,
          [
            "test"
          ],
          "192.168.1.0/24",
          "192.168.1.0/24",
          "00:00:00:00:00:00",
          [
            {
              "x": 0.0,
              "y": 0.0
            },
            {
              "x": 1.0,
              "y": 1.0
            }
          ],
          {
            "x": 0.0,
            "y": 0.0
          },
          {
            "min": {
              "x": 0.0,
              "y": 0.0
            },
            "max": {
              "x": 0.0,
              "y": 0.0
            }
          },
          "2021-01-01",
          "12:34:56.0",
          "2021-01-01T12:34:56Z",
          "00000000-0000-0000-0000-000000000000"
        ]
        "###);

        let schema = schema_for!(Value);
        assert_json_snapshot!(schema, @r###"
        {
          "$schema": "https://json-schema.org/draft/2020-12/schema",
          "title": "Value",
          "description": "A PostgreSQL value",
          "type": [
            "object",
            "array",
            "string",
            "number",
            "boolean",
            "null"
          ]
        }
        "###);

        let schema = json!({
            "type": "array",
            "items": schema.to_value()
        });
        jsonschema::validate(&schema, &serde_json::to_value(value).unwrap()).unwrap();
    }
}
