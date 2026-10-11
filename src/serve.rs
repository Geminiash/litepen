//! 轻笔 · LitePen — std 手写的极简静态服务器。
//!
//! TcpListener + 每连接一线程；正确的 MIME、样式化的 404、路径穿越防护。
//! 只为本地预览而生，绑定 127.0.0.1。

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

pub fn run(dir: &Path, port: u16) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!(
            "{} 不是一个目录。\n把信的目录交给它，例如：litepen serve out",
            dir.display()
        ));
    }
    let root: PathBuf = fs::canonicalize(dir)
        .map_err(|e| format!("无法解析目录 {}：{e}", dir.display()))?;
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| {
        format!(
            "端口 {port} 监听不上：{e}\n换一个空闲端口再试，例如：litepen serve {} -p 8080",
            dir.display()
        )
    })?;
    let addr = listener
        .local_addr()
        .map_err(|e| format!("读不到监听地址：{e}"))?;
    println!("信摆好了：http://{addr}");
    println!("按 Ctrl+C 收摊。");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let root = root.clone();
                let _ = thread::Builder::new()
                    .name("litepen-conn".into())
                    .spawn(move || handle(s, &root));
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

fn handle(mut stream: TcpStream, root: &Path) {
    let Some(request_line) = read_request(&mut stream) else {
        return;
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");
    if method != "GET" && method != "HEAD" {
        let _ = respond(&mut stream, 405, "Method Not Allowed", "text/html; charset=utf-8", &page_405(), false);
        return;
    }
    let head_only = method == "HEAD";

    let mut raw = target.split(['?', '#']).next().unwrap_or("/").to_string();
    if let Some(rest) = raw.strip_prefix("http://") {
        raw = match rest.find('/') {
            Some(i) => rest[i..].to_string(),
            None => "/".to_string(),
        };
    }
    let decoded = percent_decode(&raw);
    if decoded.contains("..") || decoded.contains('\\') || decoded.contains('\0') {
        let _ = respond(&mut stream, 404, "Not Found", "text/html; charset=utf-8", &page_404(), head_only);
        return;
    }
    let rel = decoded.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    let candidate = root.join(rel);

    let served = match fs::canonicalize(&candidate) {
        Ok(p) if p.starts_with(root) => {
            if p.is_dir() {
                fs::canonicalize(p.join("index.html")).ok()
            } else {
                Some(p)
            }
        }
        _ => None,
    };
    let Some(path) = served.filter(|p| p.is_file()) else {
        let _ = respond(&mut stream, 404, "Not Found", "text/html; charset=utf-8", &page_404(), head_only);
        return;
    };
    match fs::read(&path) {
        Ok(bytes) => {
            let ct = content_type(&path);
            let _ = respond(&mut stream, 200, "OK", ct, &bytes, head_only);
        }
        Err(_) => {
            let _ = respond(&mut stream, 404, "Not Found", "text/html; charset=utf-8", &page_404(), head_only);
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Option<String> {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        if buf.windows(4).any(|w| w == b"\r\n\r\n") || buf.len() > 8192 {
            break;
        }
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(_) => return None,
        }
    }
    String::from_utf8_lossy(&buf).lines().next().map(str::to_string)
}

fn respond(
    stream: &mut TcpStream,
    code: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    if !head_only {
        stream.write_all(body)?;
    }
    stream.flush()
}

fn content_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "txt" | "md" | "markdown" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

const PAGE_CSS: &str = "body{margin:0;min-height:100vh;display:grid;place-items:center;background:#f5f1e8;color:#3a352e;font-family:'Noto Serif SC','Songti SC',SimSun,serif;text-align:center;line-height:2}main{padding:2rem}p.num{margin:0;font-size:3rem;letter-spacing:-.02em;color:#e63946}a{color:inherit}";

fn status_page(num: &str, status_line: &str, action_line: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html lang="zh-CN"><head><meta charset="utf-8"><title>{num} · LitePen</title><style>{PAGE_CSS}</style></head><body><main><p class="num">{num}</p><p>{status_line}</p><p>{action_line}</p></main></body></html>"#
    )
}

fn page_404() -> Vec<u8> {
    status_page(
        "404",
        "这条路没有通向任何地方。",
        "<a href=\"/\">回到首页</a>，或者查一查地址是不是敲错了。",
    )
    .into_bytes()
}

fn page_405() -> Vec<u8> {
    status_page("405", "这里只听得懂 GET 和 HEAD。", "<a href=\"/\">回到首页</a>。").into_bytes()
}
