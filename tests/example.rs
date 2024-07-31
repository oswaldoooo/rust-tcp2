#[cfg(test)]
mod examples {
    #[tokio::test]
    async fn server() {
        let mut listener = tcp2::TcpListener::bind("0.0.0.0:9999".to_string())
            .await
            .expect("bind failed");
        listener
            .start(|a| async move {
                let mut buff = [0u8; 128];
                let mut lock=a.write().await;
                match lock.read(&mut buff).await {
                    Err(err) => {
                        eprintln!("read msg error {err}");
                        return;
                    }
                    Ok(size) => {
                      drop(lock);
                        if let Err(err) = a.write().await.write(&buff[0..size]).await {
                            eprintln!("write to client failed {err}");
                        }
                    }
                }
            })
            .await
            .expect("start error");
    }
    #[tokio::test]
    async fn client() {
        let id = [0x33, 0x22, 0x44, 0x55, 0x66, 0x77, 0xac, 0x01];
        let mut conn = tcp2::TcpStream::connect("127.0.0.1:9999".to_string(), id)
            .await
            .expect("connect to servre failed");
        let word = "hello world";
        conn.write(word.as_bytes())
            .await
            .expect("send msg to server failed");
        let mut buff = [0u8; 128];
        let size = conn.read(&mut buff).await.expect("read msg failed");
        println!("{}", String::from_utf8(buff[0..size].to_vec()).unwrap());
    }
}
