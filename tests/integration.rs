//! 轻笔 · LitePen — 集成测试。
//!
//! 通过 `CARGO_BIN_EXE_litepen` 直接驱动编译出的二进制，覆盖：
//! frontmatter 全字段渲染、markdown 关键元素转换、--check 输出、serve 的 MIME/404。

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_litepen")
}

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "litepen-it-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[&str]) -> (String, String, bool) {
    let out = Command::new(bin()).args(args).output().expect("启动 litepen 失败");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    )
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

// ---------- litepen new + build：frontmatter 全字段 ----------

#[test]
fn new_template_then_build_renders_all_fields() {
    let dir = tmp("fields");
    let md = dir.join("letter.md");
    let (out, err, ok) = run(&["new", md.to_str().unwrap()]);
    assert!(ok, "new 失败：{out}{err}");

    let src = read(&md);
    assert!(src.contains("title: 秋日一封信"));
    assert!(src.contains("signature: 某个人"));
    assert!(src.contains("accent: \"#e63946\""));
    assert!(src.contains("decor: true"));
    assert!(src.contains("footer: 手写于深夜与清晨之间"));
    let date = src
        .lines()
        .find(|l| l.starts_with("date:"))
        .unwrap()
        .trim_start_matches("date:")
        .trim()
        .to_string();
    assert_eq!(date.len(), 10, "date 应为 YYYY-MM-DD：{date}");

    let outdir = dir.join("out");
    let (out, err, ok) = run(&[
        "build",
        md.to_str().unwrap(),
        "-o",
        outdir.to_str().unwrap(),
    ]);
    assert!(ok, "build 失败：{out}{err}");
    let html = read(&outdir.join("index.html"));

    assert!(html.contains("<title>秋日一封信</title>"));
    assert!(html.contains(&format!(r#"<p class="letter-date">{date}</p>"#)));
    assert!(html.contains(r#"<span class="sig-name">某个人</span>"#));
    assert!(html.contains("--accent:#e63946"));
    assert!(html.contains("手写于深夜与清晨之间"));
    // 模板里的 `---` 应渲染为花枝分隔线（内联 SVG）。
    assert!(html.contains(r#"class="divider""#));
    assert!(html.contains("<svg"));
    // 自包含：无 <link>/<img>/<script src>。
    assert!(!html.contains("<link"));
    assert!(!html.contains("<img"));
    assert!(!html.contains("<script"));
}

#[test]
fn build_twice_into_existing_file_is_rejected() {
    let dir = tmp("conflict");
    let md = dir.join("letter.md");
    let (_, err, ok) = run(&["new", md.to_str().unwrap()]);
    assert!(ok, "{err}");
    let (_, err, ok) = run(&["new", md.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("不覆盖"), "{err}");
}

// ---------- 强调色：frontmatter 与 --accent 覆盖 ----------

#[test]
fn accent_from_frontmatter_and_flag() {
    let dir = tmp("accent");
    let md = dir.join("a.md");
    write_file(
        &md,
        "---\ntitle: 色\naccent: \"#2E7D32\"\n---\n\n正文。\n",
    );
    let html_path = dir.join("a.html");
    let (out, err, ok) = run(&[
        "build",
        md.to_str().unwrap(),
        "-o",
        html_path.to_str().unwrap(),
    ]);
    assert!(ok, "{out}{err}");
    assert!(read(&html_path).contains("--accent:#2e7d32"));

    let html2 = dir.join("b.html");
    let (out, err, ok) = run(&[
        "build",
        md.to_str().unwrap(),
        "-o",
        html2.to_str().unwrap(),
        "--accent",
        "#3A6EA5",
    ]);
    assert!(ok, "{out}{err}");
    assert!(read(&html2).contains("--accent:#3a6ea5"));

    let (_, err, ok) = run(&["build", md.to_str().unwrap(), "--accent", "red"]);
    assert!(!ok);
    assert!(err.contains("hex"), "{err}");
}

// ---------- --dark 夜笺变体 ----------

#[test]
fn dark_variant_switches_theme() {
    let dir = tmp("dark");
    let md = dir.join("letter.md");
    write_file(&md, "---\ntitle: 夜\n---\n\n夜里写信。\n");
    let light = dir.join("light.html");
    let night = dir.join("night.html");
    let (o, e, ok) = run(&["build", md.to_str().unwrap(), "-o", light.to_str().unwrap()]);
    assert!(ok, "{o}{e}");
    let (o, e, ok) = run(&[
        "build",
        md.to_str().unwrap(),
        "-o",
        night.to_str().unwrap(),
        "--dark",
    ]);
    assert!(ok, "{o}{e}");
    let l = read(&light);
    let n = read(&night);
    assert!(!l.contains(r#"class="night""#));
    assert!(n.contains(r#"<body class="night""#));
    assert!(n.contains("content=\"dark\""));
    assert!(n.contains("#1c1c32")); // 夜笺纸底（深靛）
    assert!(n.contains("#e8d4a6")); // 暖金墨色
}

// ---------- markdown 关键元素转换 ----------

#[test]
fn markdown_elements_convert() {
    let dir = tmp("markdown");
    let md = dir.join("letter.md");
    write_file(
        &md,
        "---\ntitle: 元素\n---\n\n## 二级标题\n\n**加粗**与*斜体*与~~删除~~与`行内码`。\n\n- 无序甲\n- 无序乙\n\n1. 有序一\n2. 有序二\n\n> 信很慢，\n> 但每一行都会到。\n\n```\ncargo build --release\n```\n\n[一个链接](https://example.com/pen)\n\n| 甲 | 乙 |\n|---|---|\n| 1 | 2 |\n",
    );
    let html_path = dir.join("index.html");
    let (out, err, ok) = run(&[
        "build",
        md.to_str().unwrap(),
        "-o",
        html_path.to_str().unwrap(),
    ]);
    assert!(ok, "{out}{err}");
    let html = read(&html_path);
    for fragment in [
        "<h2>二级标题</h2>",
        "<strong>加粗</strong>",
        "<em>斜体</em>",
        "<del>删除</del>",
        "<code>行内码</code>",
        "<ul>",
        "<li>无序甲</li>",
        "<ol>",
        "<li>有序一</li>",
        "<blockquote>",
        "信很慢，",
        "<pre>",
        "cargo build --release",
        r#"<a href="https://example.com/pen""#,
        "<table>",
        "<th>",
    ] {
        assert!(html.contains(fragment), "缺少 {fragment}，HTML 片段：{}", &html[..html.len().min(2000)]);
    }
}

// ---------- 分隔线：decor 开关 ----------

#[test]
fn hr_becomes_divider_and_decor_false_switches_to_plain() {
    let dir = tmp("decor");
    let md_on = dir.join("on.md");
    write_file(&md_on, "---\ntitle: 花\n---\n\n上一段。\n\n---\n\n下一段。\n");
    let html_on = dir.join("on.html");
    let (o, e, ok) = run(&["build", md_on.to_str().unwrap(), "-o", html_on.to_str().unwrap()]);
    assert!(ok, "{o}{e}");
    let html = read(&html_on);
    assert!(html.contains(r#"class="divider""#));
    assert!(html.contains("花枝分隔线"));

    let md_off = dir.join("off.md");
    write_file(&md_off, "---\ntitle: 素\ndecor: false\n---\n\n上一段。\n\n---\n\n下一段。\n");
    let html_off = dir.join("off.html");
    let (o, e, ok) = run(&["build", md_off.to_str().unwrap(), "-o", html_off.to_str().unwrap()]);
    assert!(ok, "{o}{e}");
    let html = read(&html_off);
    assert!(!html.contains("<svg"), "decor=false 不应再有 SVG");
    assert!(html.contains(r#"<hr class="plain">"#));
}

// ---------- footer 覆盖 ----------

#[test]
fn footer_is_overridable() {
    let dir = tmp("footer");
    let md = dir.join("letter.md");
    write_file(
        &md,
        "---\ntitle: 语\nfooter: 慢慢来，比较快。\n---\n\n正文。\n",
    );
    let html_path = dir.join("index.html");
    let (o, e, ok) = run(&["build", md.to_str().unwrap(), "-o", html_path.to_str().unwrap()]);
    assert!(ok, "{o}{e}");
    let html = read(&html_path);
    assert!(html.contains("慢慢来，比较快。"));
    assert!(!html.contains("手写于深夜与清晨之间"));
}

// ---------- --check 输出 ----------

#[test]
fn check_reports_size_and_external_links() {
    let dir = tmp("check");
    let md = dir.join("clean.md");
    write_file(&md, "---\ntitle: 净\n---\n\n一封干净的信。\n");
    let (out, err, ok) = run(&["build", md.to_str().unwrap(), "-o", dir.to_str().unwrap(), "--check"]);
    assert!(ok, "{out}{err}");
    assert!(out.contains("正文体积"), "{out}");
    assert!(out.contains("字节"), "{out}");
    assert!(out.contains("自包含"), "{out}");

    let md2 = dir.join("dirty.md");
    write_file(
        &md2,
        "---\ntitle: 杂\n---\n\n看图。\n\n<img src=\"https://example.com/pic.png\" alt=\"外链图\">\n",
    );
    let (out, err, ok) = run(&["build", md2.to_str().unwrap(), "-o", dir.join("d").to_str().unwrap(), "--check"]);
    assert!(ok, "{out}{err}");
    assert!(out.contains("外部资源引用"), "{out}");
    assert!(out.contains("https://example.com/pic.png"), "{out}");
}

// ---------- 错误处理 ----------

#[test]
fn frontmatter_and_path_errors_are_actionable() {
    let dir = tmp("errors");
    let bad = dir.join("unclosed.md");
    write_file(&bad, "---\ntitle: 没收尾\n\n正文\n");
    let (_, err, ok) = run(&["build", bad.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("---"), "{err}");

    let bad2 = dir.join("baddecor.md");
    write_file(&bad2, "---\ntitle: 怪\ndecor: maybe\n---\n\n正文\n");
    let (_, err, ok) = run(&["build", bad2.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("decor"), "{err}");

    let (_, err, ok) = run(&["build", dir.join("missing.md").to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("读不到"), "{err}");
}

// ---------- serve：MIME / 404 / 防穿越 ----------

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn http_get(port: u16, path: &str) -> (u16, String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).expect("连不上 serve");
    write!(s, "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    let status: u16 = text
        .split_whitespace()
        .nth(1)
        .expect("响应缺状态行")
        .parse()
        .expect("状态码不是数字");
    let ct = text
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-type:"))
        .map(|l| l[13..].trim().to_string())
        .unwrap_or_default();
    let body_end = text.find("\r\n\r\n").map(|i| i + 4).unwrap_or(text.len());
    (status, ct, text[body_end..].to_string())
}

#[test]
fn serve_serves_mime_404_and_blocks_traversal() {
    let dir = tmp("serve");
    write_file(&dir.join("index.html"), "<!DOCTYPE html><html><body>信之家</body></html>");
    write_file(&dir.join("style.css"), "body{color:#333}");
    write_file(&dir.join("pic.svg"), "<svg xmlns='http://www.w3.org/2000/svg'></svg>");
    write_file(&dir.join("photo.webp"), "RIFFxxxxWEBP");
    write_file(&dir.join("notes.md"), "# 笔记");

    let mut child = Command::new(bin())
        .args(["serve", dir.to_str().unwrap(), "-p", "0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("serve 启动失败");
    let stdout = child.stdout.take().expect("serve 无 stdout");
    let mut reader = BufReader::new(stdout);
    let mut port = None;
    let mut line = String::new();
    for _ in 0..100 {
        line.clear();
        let n = reader.read_line(&mut line).unwrap();
        if n == 0 {
            break;
        }
        if let Some(idx) = line.rfind("http://127.0.0.1:") {
            port = line[idx + "http://127.0.0.1:".len()..]
                .trim()
                .parse::<u16>()
                .ok();
            if port.is_some() {
                break;
            }
        }
    }
    let port = port.expect("serve 没报告端口");
    let _server = Server(child); // 测试结束自动 kill

    let (status, ct, body) = http_get(port, "/");
    assert_eq!(status, 200);
    assert!(ct.starts_with("text/html"), "{ct}");
    assert!(body.contains("信之家"));

    let cases: [(&str, &str); 5] = [
        ("/style.css", "text/css"),
        ("/pic.svg", "image/svg+xml"),
        ("/photo.webp", "image/webp"),
        ("/notes.md", "text/plain"),
        ("/index.html", "text/html"),
    ];
    for (path, mime) in cases {
        let (status, ct, _) = http_get(port, path);
        assert_eq!(status, 200, "{path}");
        assert!(ct.starts_with(mime), "{path} → {ct}（期望 {mime}）");
    }

    let (status, ct, body) = http_get(port, "/no-such-letter.html");
    assert_eq!(status, 404);
    assert!(ct.starts_with("text/html"));
    assert!(body.contains("404"));

    for evil in ["/../Cargo.toml", "/%2e%2e/Cargo.toml", "/..\\Cargo.toml"] {
        let (status, _, _) = http_get(port, evil);
        assert_ne!(status, 200, "穿越路径 {evil} 不应放行");
    }

    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(s, "POST / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    assert!(text.starts_with("HTTP/1.1 405"), "{}", &text[..40.min(text.len())]);
}

#[test]
fn build_output_direct_html_file_and_nested_dir() {
    let dir = tmp("outmode");
    let md = dir.join("letter.md");
    write_file(&md, "---\ntitle: 出口\n---\n\n正文。\n");
    let file_out = dir.join("nested").join("deep").join("letter.html");
    let (o, e, ok) = run(&["build", md.to_str().unwrap(), "-o", file_out.to_str().unwrap()]);
    assert!(ok, "{o}{e}");
    assert!(read(&file_out).contains("出口"));

    let dir_out = dir.join("site");
    let (o, e, ok) = run(&["build", md.to_str().unwrap(), "-o", dir_out.to_str().unwrap()]);
    assert!(ok, "{o}{e}");
    assert!(read(&dir_out.join("index.html")).contains("出口"));
}
