use async_std::net::TcpStream;
use async_std::task;

fn main() {
    task::block_on(async {
        let stream = TcpStream::connect("localhost:6379").await;
    })
}
