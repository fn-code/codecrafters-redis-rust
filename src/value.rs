use anyhow::Result;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    NullBulkString,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::SimpleString(s) => write!(f, "{}", s),
            Value::BulkString(s) => write!(f, "{}", s),
            _ => write!(f, ""),
        }
    }
}

impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            Value::NullBulkString => "$-1\r\n".to_string(),
            Value::Array(v) => {
                let mut serialized = format!("*{}\r\n", v.len());
                for item in v {
                    serialized.push_str(&item.serialize());
                }
                serialized
            }
        }
    }

    pub fn get_string(&self) -> Option<String> {
        match self {
            Value::SimpleString(s) => Some(s.clone()),
            Value::BulkString(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn parse_command(value: Value) -> Result<(String, Vec<Value>)> {
        match value {
            Value::Array(a) => {
                Ok((
                    unpack_bulk_str(a.first().unwrap().clone())?,
                    a.into_iter().skip(1).collect(),
                ))
            }
            Value::SimpleString(s) => {
                Ok((s, vec![]))
            }
            _ => Err(anyhow::anyhow!("Not an array"))
        }
    }


}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Not a bulk string"))
    }
}