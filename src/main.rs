use async_std::io;
use async_std::net::TcpStream;
use async_std::prelude::*;

#[async_std::main]
async fn main() -> io::Result<()> {
    // Setting up TCP connection
    let mut stream: TcpStream = TcpStream::connect("localhost:6379").await?;

    // Setting up PING command
    let command: &[u8; 14] = b"*1\r\n$4\r\nPING\r\n"; // [42, 49, 13, 10, 36, 52, 13, 10, 80, 73, 78, 71, 13, 10]
    let mut buffer: Vec<u8> = vec![0; 1024];

    stream.write_all(command).await?;
    let bytes_read: usize = stream.read(&mut buffer).await?;

    // println!("{:?}", std::str::from_utf8(&buffer[..bytes_read]));
    parse_response(&buffer[0..bytes_read]);
    Ok(())
}

fn parse_response(buffer: &[u8]) -> Result<&str, String> {
    if buffer.is_empty() {
        return Err("Empty Buffer".into());
    }

    if buffer[0] == b'-' {
        return Err(format!(
            "Error Response: {:?}",
            &buffer[1..buffer.len() - 2]
        ));
    }

    Ok(std::str::from_utf8(&buffer[1..buffer.len() - 2]).unwrap())
}
