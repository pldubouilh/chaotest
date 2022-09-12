use anyhow::{Context, Result};
use core::time;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;

use crate::fetcher::fetcher;

mod fetcher;

pub enum INSTRUCTION {
    OnceDelayWriteMs(u64),   // 1 slow request, other are normal
    AlwaysDelayWriteMs(u64), // always slow
}
//   - fails halfway once/twice...
//   - slow
//   - flake

use crate::INSTRUCTION::*;

fn handle_read(mut stream: &TcpStream) -> Result<()> {
    let mut buf = [0u8; 4096];
    let _ret = stream.read(&mut buf)?;
    // let req_str = String::from_utf8_lossy(&buf);
    // eprintln!("~~ {}", req_str);
    Ok(())
}

fn send_preamble(payload: &[u8], stream: &mut TcpStream) -> Result<()> {
    let cl = format!("content-length: {}\r\n", payload.len());
    let mut response = "HTTP/1.1 200 OK\r\n".to_string();
    response.push_str("content-type: application/octet-stream\r\n");
    response.push_str(&cl);
    response.push_str("\r\n");
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn send_payload(
    mut stream: TcpStream,
    payload: &[u8],
    delay_ms: u64,
    should_delay: bool,
) -> Result<()> {
    send_preamble(payload, &mut stream)?;

    stream.write_all(&payload[0..2])?;
    if should_delay {
        sleep(time::Duration::from_millis(delay_ms));
    }
    stream.write_all(&payload[2..])?;

    Ok(())
}

fn serve(instruction: INSTRUCTION, listener: TcpListener, payload: &[u8]) -> Result<()> {
    let mut reqs = 0;

    #[allow(clippy::explicit_counter_loop)]
    for stream in listener.incoming() {
        let stream = stream.context("err stream")?;
        handle_read(&stream).context("failed reading")?;

        match instruction {
            OnceDelayWriteMs(delay_ms) => send_payload(stream, payload, delay_ms, reqs == 0)?,
            AlwaysDelayWriteMs(delay_ms) => send_payload(stream, payload, delay_ms, true)?,
        };

        reqs += 1;
    }

    Ok(())
}

pub fn init(src: &str, instruction: INSTRUCTION) -> Result<String> {
    let mut payload = Vec::new();
    fetcher(src, &mut payload)?;

    let port = option_env!("PORT_CHAOS").unwrap_or("0");
    let ip = option_env!("URL_CHAOS").unwrap_or("127.0.0.1");
    let bindstr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(bindstr).context("cant bind")?;
    let addr = listener.local_addr().unwrap().to_string();

    let server_start = move || serve(instruction, listener, &payload);
    std::thread::spawn(server_start);

    let full_addr = format!("http://{}", addr);
    Ok(full_addr)
}
