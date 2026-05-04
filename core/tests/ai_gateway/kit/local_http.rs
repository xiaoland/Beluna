use serde_json::Value;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

#[derive(Debug)]
pub struct CapturedRequest {
    pub path: String,
    pub body: Value,
}

pub struct LocalJsonServer {
    endpoint: String,
    requests: mpsc::Receiver<CapturedRequest>,
}

impl LocalJsonServer {
    pub async fn start(responses: Vec<Value>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let (tx, rx) = mpsc::channel(responses.len().max(1));

        tokio::spawn(async move {
            for response in responses {
                let (mut socket, _) = listener.accept().await.expect("accept");
                let request = read_request(&mut socket).await;
                tx.send(request).await.expect("send captured request");
                write_json_response(&mut socket, response).await;
            }
        });

        Self {
            endpoint: format!("http://{addr}/v1"),
            requests: rx,
        }
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub async fn next_request(&mut self) -> CapturedRequest {
        self.requests.recv().await.expect("captured request")
    }
}

async fn read_request(socket: &mut TcpStream) -> CapturedRequest {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let header_end = loop {
        let n = socket.read(&mut chunk).await.expect("read request");
        assert!(n > 0, "connection closed before headers");
        buffer.extend_from_slice(&chunk[..n]);
        if let Some(index) = find_header_end(&buffer) {
            break index;
        }
    };

    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let request_line = headers.lines().next().expect("request line");
    let mut parts = request_line.split_whitespace();
    let _method = parts.next().expect("method");
    let path = parts.next().expect("path").to_string();
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().expect("content-length"))
        })
        .unwrap_or(0);

    let body_start = header_end + 4;
    while buffer.len() < body_start + content_length {
        let n = socket.read(&mut chunk).await.expect("read body");
        assert!(n > 0, "connection closed before body");
        buffer.extend_from_slice(&chunk[..n]);
    }
    let body = serde_json::from_slice(&buffer[body_start..body_start + content_length])
        .expect("json request body");

    CapturedRequest { path, body }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

async fn write_json_response(socket: &mut TcpStream, response: Value) {
    let body = response.to_string();
    let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    socket
        .write_all(response.as_bytes())
        .await
        .expect("write response");
}
