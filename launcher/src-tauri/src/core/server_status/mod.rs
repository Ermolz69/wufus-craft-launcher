use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use serde::Serialize;
use tracing::debug;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerState {
    Online,
    Offline,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub state: ServerState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_players: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ping_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl ServerStatus {
    pub fn online(players: u32, max_players: u32, ping_ms: u64, version: String) -> Self {
        Self {
            state: ServerState::Online,
            players: Some(players),
            max_players: Some(max_players),
            ping_ms: Some(ping_ms),
            version: Some(version),
        }
    }

    pub fn offline() -> Self {
        Self {
            state: ServerState::Offline,
            players: None,
            max_players: None,
            ping_ms: None,
            version: None,
        }
    }
}

/// Ping a Minecraft server using the 1.7+ Server List Ping (SLP) protocol.
///
/// Returns `Ok(ServerStatus)` on success, `Err(reason)` when the server
/// is unreachable. Caller should treat any error as Offline.
///
/// Connection and read timeouts are both 3 seconds.
pub fn ping(host: &str, port: u16) -> Result<ServerStatus, String> {
    let addr = format!("{host}:{port}");
    let socket_addr = addr
        .to_socket_addrs()
        .map_err(|e| format!("DNS lookup failed: {e}"))?
        .next()
        .ok_or_else(|| format!("No address resolved for {addr}"))?;

    let start = Instant::now();

    let mut stream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_secs(3))
            .map_err(|e| format!("Connect: {e}"))?;

    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(3))).ok();

    // 1 — Handshake (sets next-state to Status)
    stream
        .write_all(&build_handshake(host, port))
        .map_err(|e| format!("Handshake write: {e}"))?;

    // 2 — Status request
    stream
        .write_all(&build_status_request())
        .map_err(|e| format!("Status request write: {e}"))?;

    stream.flush().map_err(|e| format!("Flush: {e}"))?;

    // 3 — Read status response
    let json = read_string_packet(&mut stream)
        .map_err(|e| format!("Status response: {e}"))?;

    let ping_ms = start.elapsed().as_millis() as u64;

    debug!(
        "SLP response ({ping_ms} ms): {}",
        &json[..json.len().min(300)]
    );

    // Parse the JSON response
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| format!("JSON parse: {e}"))?;

    let players = value["players"]["online"].as_u64().unwrap_or(0) as u32;
    let max = value["players"]["max"].as_u64().unwrap_or(0) as u32;
    let version = value["version"]["name"]
        .as_str()
        .unwrap_or("Unknown")
        .to_string();

    Ok(ServerStatus::online(players, max, ping_ms, version))
}

// ── SLP packet helpers ─────────────────────────────────────────────────────

/// Encode a VarInt (little-endian 7-bit groups, MSB = continuation).
fn varint(mut value: i32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(5);
    loop {
        let mut b = (value & 0x7F) as u8;
        value = ((value as u32) >> 7) as i32;
        if value != 0 {
            b |= 0x80;
        }
        buf.push(b);
        if value == 0 {
            break;
        }
    }
    buf
}

/// Decode a VarInt from a `Read` source.
fn read_varint(r: &mut impl Read) -> io::Result<i32> {
    let mut value = 0u32;
    let mut shift = 0u32;
    loop {
        let mut b = [0u8; 1];
        r.read_exact(&mut b)?;
        value |= ((b[0] & 0x7F) as u32) << shift;
        if b[0] & 0x80 == 0 {
            return Ok(value as i32);
        }
        shift += 7;
        if shift >= 35 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VarInt too large",
            ));
        }
    }
}

/// Encode a Minecraft String: `[VarInt length][UTF-8 bytes]`.
fn mc_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut buf = varint(bytes.len() as i32);
    buf.extend_from_slice(bytes);
    buf
}

/// Wrap payload in a Minecraft packet frame: `[VarInt total_length][payload]`.
fn framed(payload: &[u8]) -> Vec<u8> {
    let mut buf = varint(payload.len() as i32);
    buf.extend_from_slice(payload);
    buf
}

fn build_handshake(host: &str, port: u16) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend(varint(0x00)); // Packet ID
    p.extend(varint(-1)); // Protocol version (-1 works for any 1.7+ server)
    p.extend(mc_string(host)); // Server address
    p.extend(port.to_be_bytes()); // Server port (big-endian u16)
    p.extend(varint(1)); // Next state: 1 = Status
    framed(&p)
}

fn build_status_request() -> Vec<u8> {
    framed(&varint(0x00)) // Packet ID 0x00, no fields
}

/// Read a single framed status-response packet and return the inner JSON string.
fn read_string_packet(r: &mut impl Read) -> io::Result<String> {
    let _frame_len = read_varint(r)?;
    let _packet_id = read_varint(r)?;
    let str_len = read_varint(r)?;

    const MAX_RESPONSE: i32 = 512 * 1024; // 512 KB sanity cap
    if str_len < 0 || str_len > MAX_RESPONSE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unexpected response length: {str_len}"),
        ));
    }

    let mut buf = vec![0u8; str_len as usize];
    r.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
