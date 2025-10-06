use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use serde_json::{json, Map, Value};
use tiberius::{Client, Config, Query, Row};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

/// Fetches data from the database and returns it as a serde_json::Value.
pub async fn fetch_data(
    connection_string: &str,
    query_sql: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let config = Config::from_ado_string(connection_string)?;

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = Client::connect(config, tcp.compat_write()).await?;

    let query = Query::new(query_sql);
    let stream = query.query(&mut client).await?;
    let rows = stream.into_first_result().await?;

    let mut results = Vec::new();

    for row in rows {
        results.push(row_to_json(row)?);
    }

    Ok(json!(results))
}

/// Converts a Tiberius row into a serde_json::Value object.
fn row_to_json(row: Row) -> Result<Value, Box<dyn std::error::Error>> {
    let mut map = Map::new();

    for (i, column) in row.columns().iter().enumerate() {
        let col_name = column.name().to_string();
        let val = match column.column_type() {
            tiberius::ColumnType::Null => Value::Null,
            tiberius::ColumnType::Bit => row.get::<bool, _>(i).map_or(Value::Null, Value::from),
            tiberius::ColumnType::Int1 | tiberius::ColumnType::Int2 | tiberius::ColumnType::Int4 => {
                row.get::<i32, _>(i).map_or(Value::Null, |v| json!(v))
            }
            tiberius::ColumnType::Int8 => row.get::<i64, _>(i).map_or(Value::Null, |v| json!(v)),
            tiberius::ColumnType::Float4 => row.get::<f32, _>(i).map_or(Value::Null, |v| json!(v)),
            tiberius::ColumnType::Float8 => row.get::<f64, _>(i).map_or(Value::Null, |v| json!(v)),
            tiberius::ColumnType::Numericn | tiberius::ColumnType::Decimaln | tiberius::ColumnType::Money | tiberius::ColumnType::Money4 => {
                row.get::<Decimal, _>(i).map_or(Value::Null, |v| json!(v))
            }
            tiberius::ColumnType::Datetime | tiberius::ColumnType::Datetime2 | tiberius::ColumnType::Datetime4 => {
                row.get::<NaiveDateTime, _>(i).map_or(Value::Null, |v| json!(v))
            }
            tiberius::ColumnType::Daten => {
                row.get::<NaiveDate, _>(i).map_or(Value::Null, |v| json!(v))
            }
            tiberius::ColumnType::Timen => {
                row.get::<NaiveTime, _>(i).map_or(Value::Null, |v| json!(v))
            }
            tiberius::ColumnType::BigChar
            | tiberius::ColumnType::BigVarChar
            | tiberius::ColumnType::NChar
            | tiberius::ColumnType::NVarchar
            | tiberius::ColumnType::Text
            | tiberius::ColumnType::NText => {
                row.get::<&str, _>(i).map_or(Value::Null, |v| json!(v))
            }
            // For simplicity, we'll default other types to Null for now.
            _ => Value::Null,
        };
        map.insert(col_name, val);
    }

    Ok(Value::Object(map))
}