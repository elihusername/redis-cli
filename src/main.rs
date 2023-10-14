use async_std::io;
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::prelude::*;

#[async_std::main]
async fn main() -> io::Result<()> {
    let mut tcp_client = Client::new("localhost:6379").await?;
    tcp_client
        .set("Elihu".into(), "Miami".into())
        .await
        .unwrap();
    println!("{}", tcp_client.get("Elihu".into()).await.unwrap());
    Ok(())
}

fn parse_response(buffer: &[u8]) -> Result<&str, Error> {
    if buffer.is_empty() {
        return Err(Error {});
    }

    if buffer[0] == b'-' {
        return Err(Error {});
    }

    Ok(std::str::from_utf8(&buffer[1..buffer.len() - 2]).unwrap())
}

struct Client {
    stream: TcpStream,
}

impl Client {
    async fn new<A: ToSocketAddrs>(addr: A) -> Result<Client, io::Error> {
        let stream: TcpStream = TcpStream::connect(addr).await?;
        Ok(Client { stream })
    }
}

impl Client {
    async fn get(&mut self, key: String) -> Result<String, Error> {
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

    async fn set(&mut self, key: String, value: String) -> Result<(), Error> {
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
struct Error {}

impl std::convert::From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Error {}
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
