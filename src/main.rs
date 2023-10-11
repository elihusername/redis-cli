use async_std::net::TcpStream;

#[async_std::main]
async fn main() {
    let stream = TcpStream::connect("localhost:6379").await;
}
