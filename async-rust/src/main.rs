use client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("127.0.0.1:11211", false).await.unwrap();
    client.set("foo", b"bar", 0, 0).await.unwrap();
    Ok(())
}
