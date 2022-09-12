use anyhow::{Context, Result};
use core::time;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;

use crate::fetcher::fetcher;

mod fetcher;

pub enum INSTRUCTION {
    OnceDelayWriteMs(u64),
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

fn handle_write(mut stream: TcpStream, payload: &[u8], delay_ms: u64) -> Result<()> {
    let cl = format!("content-length: {}\r\n", payload.len());

    let mut response = "HTTP/1.1 200 OK\r\n".to_string();
    response.push_str("content-type: application/octet-stream\r\n");
    response.push_str(&cl);
    response.push_str("\r\n\r\n");

    stream.write_all(response.as_bytes())?;
    if delay_ms > 0 {
        stream.write_all(&payload[0..2])?;
        sleep(time::Duration::from_millis(delay_ms));
        stream.write_all(&payload[2..])?;
    } else {
        stream.write_all(payload)?;
    }
    Ok(())
}

fn serve(instruction: INSTRUCTION, listener: TcpListener, payload: &[u8]) -> Result<()> {
    let (mut count, delay_ms) = match instruction {
        OnceDelayWriteMs(delay_ms) => (1, delay_ms),
    };

    for stream in listener.incoming() {
        let stream = stream.context("err stream")?;
        handle_read(&stream).context("failed reading")?;
        handle_write(stream, payload, delay_ms).context("failed write")?;

        count -= 1;
        if count == 0 {
            break;
        }
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
