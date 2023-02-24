use iouring::UringClient;

fn main() {
    let (key, val) = ("foo", "bar");

    tokio_uring::start(async {
        let mut client = UringClient::connect("/var/run/memcached/memcached.sock").await.unwrap();
        client.set(key, val.as_bytes(), 0, 0).await.unwrap();

        let v = client.get("foo").await.unwrap().unwrap();
        assert_eq!(v, val.as_bytes());
    });
}
