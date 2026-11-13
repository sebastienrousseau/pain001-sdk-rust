// SPDX-License-Identifier: Apache-2.0 OR MIT
//! A one-shot HTTP server on loopback for unit tests: it answers one
//! request with a canned response and hands the request back.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver};
use std::thread;

/// The request the stub received.
#[derive(Debug)]
pub(crate) struct Seen {
    pub request_line: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Seen {
    /// The first header with this (lower-case) name.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Starts a stub answering `status` with `body`; returns its base URL and
/// a receiver for the request it saw.
pub(crate) fn serve(status: u16, body: &str) -> (String, Receiver<Seen>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let url = format!("http://{}", listener.local_addr().expect("local addr"));
    let response = format!(
        "HTTP/1.1 {status} Stub\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let seen = read_request(&mut reader);
        let mut stream = stream;
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        let _ = sender.send(seen);
    });
    (url, receiver)
}

fn read_request(reader: &mut impl BufRead) -> Seen {
    let mut request_line = String::new();
    reader.read_line(&mut request_line).expect("request line");
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("header line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.push((key.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }
    let length = headers
        .iter()
        .find(|(key, _)| key == "content-length")
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).expect("body");
    Seen {
        request_line: request_line.trim_end().to_owned(),
        headers,
        body: String::from_utf8(body).expect("utf-8 body"),
    }
}
