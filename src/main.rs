use async_std::io;
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::prelude::*;

// TODO:
// Be able to type out commands

#[async_std::main]
async fn main() -> Result<(), RedisError> {
    let mut tcp_client = Client::new("localhost:6379").await?;
    tcp_client.set("Elihu".into(), "Miami".into()).await?;
    println!("{}", tcp_client.get("Elihu".into()).await?);
    Ok(())
}

fn parse_response(buffer: &[u8]) -> Result<&str, RedisError> {
    if buffer.is_empty() {
        return Err(RedisError::EmptyBuffer);
    }

    match buffer.first() {
        Some(&b'-') => {
            let utf8_result: Result<&str, std::str::Utf8Error> =
                std::str::from_utf8(&buffer[1..buffer.len() - 2]);
            let valid_utf8: &str = utf8_result.map_err(|_| RedisError::Utf8DecodingError)?;
            Err(RedisError::ResponseError(valid_utf8.into()))
        }
        Some(&b'$') => {
            let newline: usize = buffer.iter().position(|&x| x == 10).unwrap_or(buffer.len());

            let utf8_result: Result<&str, std::str::Utf8Error> =
                std::str::from_utf8(&buffer[newline + 1..buffer.len() - 2]);
            let valid_utf8: &str = utf8_result.map_err(|_| RedisError::Utf8DecodingError)?;
            Ok(valid_utf8)
        }
        _ => {
            let utf8_result: Result<&str, std::str::Utf8Error> =
                std::str::from_utf8(&buffer[1..buffer.len() - 2]);
            let valid_utf8: &str = utf8_result.map_err(|_| RedisError::Utf8DecodingError)?;
            Ok(valid_utf8)
        }
    }
}

struct Client {
    stream: TcpStream,
}

impl Client {
    async fn new<A: ToSocketAddrs>(addr: A) -> Result<Client, RedisError> {
        let stream: TcpStream = TcpStream::connect(addr).await?;
        Ok(Client { stream })
    }
}

impl Client {
    async fn get(&mut self, key: String) -> Result<String, RedisError> {
        let mut buffer: Vec<u8> = vec![];
        let command: RespValues = RespValues::Array(vec![
            RespValues::BulkString(b"GET".to_vec()),
            RespValues::BulkString(key.into_bytes()),
        ]);

        command.serialize(&mut buffer);
        self.stream.write_all(&buffer).await?;

        let bytes_read = self.stream.read(&mut buffer).await?;
        let resp: &str = parse_response(&buffer[..bytes_read])?;

        Ok(resp.to_owned())
    }

    async fn set(&mut self, key: String, value: String) -> Result<(), RedisError> {
        let mut buffer: Vec<u8> = vec![];
        let command: RespValues = RespValues::Array(vec![
            RespValues::BulkString(b"SET".to_vec()),
            RespValues::BulkString(key.into_bytes()),
            RespValues::BulkString(value.into_bytes()),
        ]);

        command.serialize(&mut buffer);
        self.stream.write_all(&buffer).await?;

        let bytes_read = self.stream.read(&mut buffer).await?;
        parse_response(&buffer[..bytes_read])?;

        Ok(())
    }
}

#[derive(Debug)]
enum RedisError {
    ConnectionError(io::Error),
    ResponseError(String),
    Utf8DecodingError,
    EmptyBuffer,
}

impl std::error::Error for RedisError {}

impl std::fmt::Display for RedisError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            RedisError::ConnectionError(io_err) => {
                write!(f, "Connection error: {}", io_err)
            }
            RedisError::ResponseError(response_err) => {
                write!(f, "Response error: {}", response_err)
            }
            RedisError::Utf8DecodingError => write!(f, "UTF8 Decoding error"),
            RedisError::EmptyBuffer => write!(f, "Buffer is empty"),
        }
    }
}

impl From<io::Error> for RedisError {
    fn from(io_error: io::Error) -> Self {
        RedisError::ConnectionError(io_error)
    }
}

enum RespValues {
    BulkString(Vec<u8>),
    Array(Vec<RespValues>),
}

impl RespValues {
    fn serialize(self, buff: &mut Vec<u8>) -> &mut Vec<u8> {
        match self {
            RespValues::Array(values) => {
                buff.push(b'*');
                buff.append(&mut format!("{}", values.len()).into_bytes());
                buff.push(b'\r');
                buff.push(b'\n');
                for value in values {
                    value.serialize(buff);
                }
            }
            RespValues::BulkString(mut data) => {
                buff.push(b'$');
                buff.append(&mut format!("{}", data.len()).into_bytes());
                buff.push(b'\r');
                buff.push(b'\n');
                buff.append(&mut data);
                buff.push(b'\r');
                buff.push(b'\n');
            }
            _ => unimplemented!(),
        }

        buff
    }
}
