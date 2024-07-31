use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[allow(dead_code)]
pub struct TcpStream {
    conn: Mutex<Option<Arc<Mutex<tokio::net::TcpStream>>>>,
    wbuff: Vec<u8>,
    rbuff: Vec<u8>,
    _close: RwLock<bool>,
    _notify: tokio::sync::Notify,
}
impl TcpStream {
    #[allow(dead_code)]
    pub fn new(conn: tokio::net::TcpStream) -> Self {
        Self {
            conn: Mutex::new(Some(Arc::new(Mutex::new(conn)))),
            wbuff: Vec::new(),
            rbuff: Vec::new(),
            _close: RwLock::new(false),
            _notify: tokio::sync::Notify::new(),
        }
    }
    #[allow(dead_code)]
    pub async fn modify(&mut self, conn: tokio::net::TcpStream) {
        let mut l = self.conn.lock().await;
        if l.is_some() {
            l.take();
        }
        *l = Some(Arc::new(Mutex::new(conn)));
        self._notify.notify_waiters();
    }
    #[allow(dead_code)]
    pub async fn connect(addr: String, id: [u8; 8]) -> Result<Self, String> {
        match tokio::net::TcpStream::connect(addr).await {
            Err(err) => {
                return Err(format!("{err}"));
            }
            Ok(mut conn) => {
                // println!("send id");
                if let Err(err) = conn.write(&id).await {
                    return Err(format!("{err}"));
                }
                // println!("return");
                return Ok(TcpStream {
                    conn: Mutex::new(Some(Arc::new(Mutex::new(conn)))),
                    wbuff: Vec::new(),
                    rbuff: Vec::new(),
                    _close: RwLock::new(false),
                    _notify: tokio::sync::Notify::new(),
                });
            }
        }
    }
    pub async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.conn.lock().await.is_none() {
            if *self._close.read().await {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "connection closed",
                ));
            }
            self._notify.notified().await;
        }
        let mut con = self.conn.lock().await;
        let tcpcon = con.take().unwrap();
        let mut _err = None;
        let mut _size = 0;
        match tcpcon.lock().await.read(buf).await {
            Ok(size) => {
                _size = size;
            }
            Err(err) => {
                _err = Some(err);
            }
        }
        *con = Some(tcpcon);
        if let Some(err) = _err {
            return Err(err);
        }
        return Ok(_size);
    }
    pub async fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.conn.lock().await.is_none() {
            if *self._close.read().await {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "connection closed",
                ));
            }
            self._notify.notified().await;
            if *self._close.read().await {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "connection closed",
                ));
            }
        }
        let mut con = self.conn.lock().await;
        let tcpcon = con.take().unwrap();
        // let _err=None;
        let mut _err = None;
        let mut _size = 0;
        match tcpcon.lock().await.write(buf).await {
            Ok(size) => {
                _size = size;
            }
            Err(err) => {
                _err = Some(err);
            }
        }
        *con = Some(tcpcon);
        if let Some(err) = _err {
            return Err(err);
        }
        return Ok(_size);
    }
}

#[allow(dead_code)]
pub struct TcpListener {
    listener: tokio::net::TcpListener,
    conn_map: BTreeMap<String, Arc<RwLock<TcpStream>>>,
}
#[allow(dead_code)]
enum MessageType {
    Ping = 0x1,
    Message,
}
impl TcpListener {
    #[allow(dead_code)]
    pub async fn bind(addr: String) -> Result<Self, String> {
        match tokio::net::TcpListener::bind(addr).await {
            Err(err) => {
                return Err(format!("{err}"));
            }
            Ok(l) => Ok(Self {
                listener: l,
                conn_map: BTreeMap::new(),
            }),
        }
    }
    #[allow(dead_code)]
    pub async fn start<F>(&mut self, f: impl Fn(Arc<RwLock<TcpStream>>) -> F) -> Result<(), String>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        // println!("start bind");
        loop {
            let conn = self.listener.accept().await;
            if let Err(err) = conn {
                return Err(format!("accept conn failed {err}"));
            }
            let (conn, _) = conn.unwrap();
            // println!("start handshake");
            match self.handshake(conn).await {
                Ok(conn) => {
                    // println!("into spawn");
                    tokio::spawn(f(conn));
                }
                _ => {}
            }
        }
    }
    async fn handshake(
        &mut self,
        mut conn: tokio::net::TcpStream,
    ) -> Result<Arc<RwLock<TcpStream>>, ()> {
        let mut buff = [0u8; 8];
        if let Err(_) = conn.read_exact(&mut buff).await {
            return Err(());
        }
        let hsh = hex::encode(buff);
        let ans: Arc<RwLock<TcpStream>>;
        if self.conn_map.contains_key(&hsh) {
            self.conn_map
                .get_mut(&hsh)
                .unwrap()
                .write()
                .await
                .modify(conn)
                .await;
            ans = self.conn_map[&hsh].clone();
        } else {
            ans = Arc::new(RwLock::new(TcpStream::new(conn)));
            self.conn_map.insert(hsh, ans.clone());
        }
        return Ok(ans);
    }
}
