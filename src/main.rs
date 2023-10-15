use async_std::io;
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::prelude::*;

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

    if buffer[0] == b'-' {
        return Err(RedisError::ResponseError);
    }
    // TODO: remove utf8 error unwrap
    // let utf8_result = std::str::from_utf8(&buffer[1..buffer.len() - 2]);
    // let valid_utf8 = utf8_result.map_err(|_| RedisError::Utf8DecodingError)?;

    Ok(std::str::from_utf8(&buffer[1..buffer.len() - 2]).unwrap())
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
    ResponseError,
    EmptyBuffer,
}

impl std::error::Error for RedisError {}

impl std::fmt::Display for RedisError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            RedisError::ConnectionError(io_err) => {
                write!(f, "Connection error: {}", io_err)
            }
            RedisError::ResponseError => write!(f, "Response error"),
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
    SimpleString(String),
    Error(Vec<u8>),
    Integer(i64),
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
