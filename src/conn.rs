use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use bytes::BytesMut;
use anyhow::{Error, Result};
use crate::value::Value;

#[derive(Debug)]
pub struct Connnection {
    stream: TcpStream,
    buffer: BytesMut
}

impl Connnection {
    pub fn new(stream: TcpStream) -> Self {
        return Connnection {
            stream,
            buffer: BytesMut::with_capacity(1024)
        }
    }

    pub async fn read_value(&mut self) -> Result<Value, Error> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        if bytes_read == 0 {
            return Ok(Value::NullBulkString);
        }

        let (v, _) = parse_message(self.buffer.split())?;
        Ok(v)
    }

    pub async fn write_value(&mut self, value: Value) -> Result<(), Error> {
        self.stream.write(value.serialize().as_bytes()).await?;
        Ok(())
    }

    pub async fn write(&mut self,  buffer: &[u8]) -> Result<()> {
        self.stream.write(buffer).await?;
        Ok(())
    }
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow::anyhow!("Not a known value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();

        return Ok((Value::SimpleString(string), len + 1))
    }

    return Err(anyhow::anyhow!("invalid string {:?}", buffer))
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (array_length, mut bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let array_length = parse_int(line)?;
        (array_length, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };

    let mut items = vec![];
    for _ in 0..array_length {
        let (value, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;
        items.push(value);
        bytes_consumed += len;
    }

    return Ok((Value::Array(items), bytes_consumed))
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_str_len = parse_int(line)?;
        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid bulk string format {:?}", buffer));
    };

    let end_of_buk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_of_buk_str + 2;

    Ok((Value::BulkString(String::from_utf8(buffer[bytes_consumed..end_of_buk_str].to_vec())?), total_parsed))
}
fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for index in 1..buffer.len() {
        if buffer[index - 1] == b'\r' && buffer[index] == b'\n'{
            return Some((&buffer[0..(index - 1)], index + 1));
        }
    }
    None
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}