use async_std::io;
use async_std::net::TcpStream;
use async_std::prelude::*;

#[async_std::main]
async fn main() -> io::Result<()> {
    // Setting up TCP connection
    let mut stream: TcpStream = TcpStream::connect("localhost:6379").await?;

    // Setting up PING command
    let command: &[u8; 14] = b"*1\r\n$4\r\nPING\r\n";
    let mut buffer: Vec<u8> = vec![0; 1024];

    stream.write_all(command).await?;
    let bytes_read: usize = stream.read(&mut buffer).await?;

    println!("{:?}", std::str::from_utf8(&buffer[..bytes_read]));
    Ok(())
}
