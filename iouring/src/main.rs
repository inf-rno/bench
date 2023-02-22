use iouring::Client;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (key, val) = ("foo", "bar");

    tokio_uring::start(async {
        let mut client = Client::connect("127.0.0.1:11211", false).await.unwrap();
        client.set(key, val.as_bytes(), 0, 0).await.unwrap();

        let v = client.get("foo").await.unwrap().unwrap();
        assert_eq!(v, val.as_bytes());

        Ok(())
    });
}
