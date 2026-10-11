//! 轻笔 · LitePen — 命令行接口与命令实现。

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::render;
use crate::serve;

#[derive(Parser)]
#[command(
    name = "litepen",
    version,
    about = "一份 Markdown 进，一封像信的 HTML 出。",
    long_about = "轻笔 · LitePen —— 把一份 Markdown 渲染成单文件自包含的静态 HTML 信笺。\n无外链、无图片依赖，CSS 全内联；--dark 出夜笺，--accent 换强调色。"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// 生成带 frontmatter 的信笺模板
    New {
        /// 要创建的 .md 文件路径
        file: PathBuf,
    },
    /// 渲染为单个自包含的 index.html
    Build {
        /// 信笺的 .md 源文件
        file: PathBuf,
        /// 输出目录（或直接给 .html 文件路径），默认 out/
        #[arg(short, long, value_name = "DIR|FILE")]
        output: Option<PathBuf>,
        /// 生成暗色「夜笺」变体
        #[arg(long)]
        dark: bool,
        /// 覆盖强调色，hex 写法，如 #E63946
        #[arg(long, value_name = "HEX")]
        accent: Option<String>,
        /// 报告正文体积与是否含外部资源引用
        #[arg(long)]
        check: bool,
    },
    /// 用极简静态服务器预览一个目录
    Serve {
        /// 要服务的目录
        dir: PathBuf,
        /// 端口，0 表示交给系统挑一个空闲的
        #[arg(short, long, default_value_t = 4141)]
        port: u16,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}

impl Cli {
    pub fn run(self) -> Result<(), String> {
        match self.command {
            Command::New { file } => new_letter(&file),
            Command::Build {
                file,
                output,
                dark,
                accent,
                check,
            } => build_letter(&file, output, dark, accent.as_deref(), check),
            Command::Serve { dir, port } => serve::run(&dir, port),
        }
    }
}

fn new_letter(file: &Path) -> Result<(), String> {
    if file.exists() {
        return Err(format!(
            "{} 已经在了，不覆盖它。\n换一个文件名，或者先腾走旧的。",
            file.display()
        ));
    }
    if let Some(parent) = file.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建目录 {}：{e}", parent.display()))?;
        }
    }
    let src = render::template(&today());
    fs::write(file, src).map_err(|e| format!("无法写入 {}：{e}", file.display()))?;
    println!("信笺备好：{}", file.display());
    println!("把想说的话填进去，然后运行：litepen build {}", file.display());
    Ok(())
}

fn build_letter(
    file: &Path,
    output: Option<PathBuf>,
    dark: bool,
    accent: Option<&str>,
    check: bool,
) -> Result<(), String> {
    let src = fs::read_to_string(file)
        .map_err(|e| format!("读不到 {}：{e}\n确认路径拼写无误后，再试一次。", file.display()))?;
    let (mut meta, body) = render::parse_source(&src)?;
    if let Some(hex) = accent {
        meta.accent = render::normalize_accent(hex)?;
    }
    let html = render::render_html(&meta, &body, dark)?;

    let out_path = match output {
        Some(p) if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("html")) => p,
        Some(p) => {
            fs::create_dir_all(&p)
                .map_err(|e| format!("无法创建输出目录 {}：{e}", p.display()))?;
            p.join("index.html")
        }
        None => {
            fs::create_dir_all("out").map_err(|e| format!("无法创建输出目录 out：{e}"))?;
            PathBuf::from("out").join("index.html")
        }
    };
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("无法创建输出目录 {}：{e}", parent.display()))?;
        }
    }
    fs::write(&out_path, &html).map_err(|e| format!("无法写入 {}：{e}", out_path.display()))?;

    println!("信写好了：{}", out_path.display());
    println!(
        "正文体积：{}（{} 字节）",
        render::human_size(html.len() as u64),
        html.len()
    );
    if check {
        let findings = render::audit_external(&html);
        if findings.is_empty() {
            println!("外链检查：未发现外部资源引用，这封信是自包含的。");
        } else {
            println!("外链检查：发现 {} 处疑似外部资源引用：", findings.len());
            for f in &findings {
                println!("  - {f}");
            }
            println!("寄出去之后这些资源可能加载不到，建议内联或删掉。");
        }
    }
    Ok(())
}

/// 今天的日期（UTC，形如 2026-10-11）。不引额外 crate，用天数换算民用历。
fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant 的 civil_from_days 算法：days since epoch → (y, m, d)。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
