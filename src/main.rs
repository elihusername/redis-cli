use async_std::io;
use async_std::net::TcpStream;
use async_std::prelude::*;

#[async_std::main]
async fn main() -> io::Result<()> {
    // Setting up TCP connection
    let mut stream: TcpStream = TcpStream::connect("localhost:6379").await?;

    // Setting up PING command
    let mut buffer = vec![];
    let command: RespValues = RespValues::Array(vec![RespValues::BulkString(b"PING".to_vec())]);

    command.serialize(&mut buffer);

    println!("{:?}", buffer);

    stream.write_all(&buffer).await?;

    let bytes_read: usize = stream.read(&mut buffer).await?;

    // println!("{:?}", std::str::from_utf8(&buffer[..bytes_read]));
    println!("{:?}", parse_response(&buffer[0..bytes_read]));
    Ok(())
}

fn parse_response(buffer: &[u8]) -> Result<&str, String> {
    if buffer.is_empty() {
        return Err("Empty Buffer".into());
    }

    if buffer[0] == b'-' {
        return Err(format!(
            "Error Response: {:?}",
            std::str::from_utf8(&buffer[1..buffer.len() - 2])
        ));
    }

    Ok(std::str::from_utf8(&buffer[1..buffer.len() - 2]).unwrap())
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
