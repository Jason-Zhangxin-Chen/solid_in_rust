use std::io::{self, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;

use serde::Deserialize;

// ── Wire protocol ────────────────────────────────────────────────────────────
//
//  Each message on the wire is laid out as:
//
//   ┌────────────┬─────────────────┬─────────────────────────┐
//   │  id: u32   │ payload_len: u32│   payload: [u8; ...]    │
//   │  (4 bytes) │   (4 bytes)     │  (payload_len bytes)    │
//   └────────────┴─────────────────┴─────────────────────────┘
//
//  All multi-byte integers are little-endian.

const MSG_HEADER_SIZE: usize = 8; // 4 (id) + 4 (payload_len)

// ── Message types ────────────────────────────────────────────────────────────

const MSG_ID_ON_BOARD_GAME: u32 = 1;

#[derive(Debug, Deserialize)]
struct OnBoardGame {
    user_id: u128,
    game_id: u128,
}

// ── Parsed message envelope ──────────────────────────────────────────────────

#[derive(Debug)]
struct Msg {
    id: u32,
    payload: Vec<u8>,
}

// ── Reading helpers ──────────────────────────────────────────────────────────

/// Fill `buf` completely from `stream`, returning an error if the connection
/// closes before all bytes arrive.
fn read_exact(stream: &mut TcpStream, buf: &mut [u8]) -> io::Result<()> {
    let mut total = 0;
    while total < buf.len() {
        match stream.read(&mut buf[total..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "connection closed by peer",
                ))
            }
            Ok(n) => total += n,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Read one `Msg` from the stream (blocking until a complete message arrives).
fn read_msg(stream: &mut TcpStream) -> io::Result<Msg> {
    let mut header = [0u8; MSG_HEADER_SIZE];
    read_exact(stream, &mut header)?;

    let id = u32::from_le_bytes(header[0..4].try_into().unwrap());
    let payload_len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;

    let mut payload = vec![0u8; payload_len];
    read_exact(stream, &mut payload)?;

    Ok(Msg { id, payload })
}

// ── Message dispatcher ───────────────────────────────────────────────────────

fn process_msg(msg: Msg) {
    match msg.id {
        MSG_ID_ON_BOARD_GAME => handle_on_board_game(msg.payload),
        unknown => eprintln!("[warn] unknown message id: {}", unknown),
    }
}

fn handle_on_board_game(payload: Vec<u8>) {
    match serde_json::from_slice::<OnBoardGame>(&payload) {
        Ok(evt) => {
            println!(
                "[OnBoardGame] user_id={} game_id={}",
                evt.user_id, evt.game_id
            );
            // TODO: add your game-onboarding logic here
        }
        Err(e) => eprintln!("[error] failed to deserialize OnBoardGame: {}", e),
    }
}

// ── Per-connection loop ──────────────────────────────────────────────────────

fn handle_connection(mut stream: TcpStream) {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());

    println!("[+] connected: {}", peer);

    loop {
        match read_msg(&mut stream) {
            Ok(msg) => process_msg(msg),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                println!("[-] disconnected: {}", peer);
                break;
            }
            Err(e) => {
                eprintln!("[error] read error from {}: {}", peer, e);
                break;
            }
        }
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let addr = "0.0.0.0:7878";
    let listener = TcpListener::bind(addr)?;
    println!("[server] listening on {}", addr);

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                thread::spawn(move || handle_connection(stream));
            }
            Err(e) => eprintln!("[error] accept failed: {}", e),
        }
    }

    Ok(())
}
