#[cfg(test)]
mod tcp2_test {
    use std::sync::Arc;
    // use TcpStream;
    use tokio::sync::RwLock;
    // extern crate tcp2;
    use tcp2::TcpStream;
    use tcp2::TcpListener;
    #[tokio::test]
    async fn test_tcp2() {
        let _ = TcpListener::bind("0.0.0.0:1999".to_string())
            .await
            .unwrap()
            .start(hand)
            .await;
    }
    async fn hand(c: Arc<RwLock<TcpStream>>) {
        let mut buff = [0u8; 128];
        println!("read msg");
        let size = c
            .write()
            .await
            .read(&mut buff)
            .await
            .expect("read msg error");
        assert!(size>0,"write msg failed,handshake failed");
        let word = "hello";
        c.write()
            .await
            .write(word.as_bytes())
            .await
            .expect("write msg failed");
    }
    #[tokio::test]
    async fn test_client() {
        let id = [0x12, 0x33, 0x44, 0x55, 0x32, 0x45, 0x45, 0x88];
        println!("prepare connect");
        let mut conn = TcpStream::connect("127.0.0.1:1999".to_string(), id)
            .await
            .expect("get connection failed");
        let word = "hello tcp2 server";
        conn.write(word.as_bytes()).await.expect("send msg failed");
        let mut buff = [0u8; 100];
        let size = conn.read(&mut buff).await.expect("read msg failed");
        assert!(size>0,"read msg failed");
    }
}
