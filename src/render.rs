//! 轻笔 · LitePen — 排版与渲染核心。
//!
//! 职责：frontmatter 解析、Markdown → HTML、内联 CSS 排版系统、外链审计。
//! 视觉规格依据 docs/visual-guide.md 第 9 节（轻笔信笺）与第 11 节速查表：
//! 纸底 #faf8f2 → #f5f1e8，墨色 #3a352e，落款章色 #E63946，圆角 4px，行高 2em。

use pulldown_cmark::{html, Options, Parser};

/// 页脚寄语默认文案（style-guide 4.4 语气），frontmatter `footer` 可覆盖。
pub const DEFAULT_FOOTER: &str = "手写于深夜与清晨之间，感谢读到这一行。";

const LIGHT_THEME_COLOR: &str = "#faf8f2";
const NIGHT_THEME_COLOR: &str = "#131324";

/// 花枝分隔线（内联 SVG，随 --accent 变色）。
/// 线稿风：勾线花瓣 + 双侧渐隐细线（shoka divider 手法）。
const DIVIDER_SVG: &str = concat!(
    r#"<div class="divider" role="separator"><svg viewBox="0 0 160 32" width="128" height="26" role="img" aria-label="花枝分隔线" xmlns="http://www.w3.org/2000/svg" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">"#,
    r#"<ellipse cx="80" cy="8.6" rx="3.1" ry="5.4"/>"#,
    r#"<ellipse cx="80" cy="8.6" rx="3.1" ry="5.4" transform="rotate(72 80 16)"/>"#,
    r#"<ellipse cx="80" cy="8.6" rx="3.1" ry="5.4" transform="rotate(144 80 16)"/>"#,
    r#"<ellipse cx="80" cy="8.6" rx="3.1" ry="5.4" transform="rotate(216 80 16)"/>"#,
    r#"<ellipse cx="80" cy="8.6" rx="3.1" ry="5.4" transform="rotate(288 80 16)"/>"#,
    r#"<circle cx="80" cy="16" r="2.1" fill="currentColor" stroke="none"/>"#,
    r#"<path d="M77 18 C 66 21 55 17 43 22 C 37 24.4 30 24 25.5 21.8"/>"#,
    r#"<path d="M83 18 C 94 21 105 17 117 22 C 123 24.4 130 24 134.5 21.8"/>"#,
    r#"<ellipse cx="57" cy="16.6" rx="4.6" ry="1.7" transform="rotate(-16 57 16.6)"/>"#,
    r#"<ellipse cx="103" cy="16.6" rx="4.6" ry="1.7" transform="rotate(16 103 16.6)"/>"#,
    r#"<circle cx="24.5" cy="21.4" r="1.7" fill="currentColor" stroke="none"/>"#,
    r#"<circle cx="135.5" cy="21.4" r="1.7" fill="currentColor" stroke="none"/>"#,
    r#"</svg></div>"#,
);

/// 全部内联 CSS。强调色不进这里 —— 以 `<body style="--accent:…">` 注入。
const CSS: &str = r#"
:root{--paper-1:#faf8f2;--paper-2:#f5f1e8;--backdrop:#eae3d4;--edge:rgba(58,53,46,.14);
--ink:#3a352e;--ink-soft:#6f6759;--ink-faint:rgba(58,53,46,.45);--hairline:rgba(58,53,46,.24);
--rule:rgba(58,53,46,.055);--code-bg:rgba(58,53,46,.06);--accent:#e63946;
--serif:"Noto Serif SC","Songti SC","Source Han Serif SC",SimSun,serif;
--mono:"SF Mono",Consolas,Menlo,monospace}
body.night{--paper-1:#1c1c32;--paper-2:#131324;--backdrop:#0b0b16;--edge:rgba(231,212,166,.16);
--ink:#e8d4a6;--ink-soft:#c8b287;--ink-faint:rgba(232,212,166,.45);--hairline:rgba(232,212,166,.26);
--rule:rgba(232,212,166,.05);--code-bg:rgba(232,212,166,.08)}
*{box-sizing:border-box}
body{margin:0;min-height:100vh;display:flex;flex-direction:column;align-items:center;
padding:clamp(1rem,4vw,3rem) clamp(.8rem,4vw,2rem);background:var(--backdrop);color:var(--ink);
font-family:var(--serif);font-size:1rem;line-height:2;-webkit-text-size-adjust:100%}
::selection{background:var(--accent);color:#fff}
.paper{width:100%;max-width:38rem;
background:radial-gradient(ellipse 80% 40% at 50% -10%,rgba(255,220,180,.10),transparent),
linear-gradient(var(--paper-1),var(--paper-2));
border:1px solid var(--edge);border-radius:4px;box-shadow:0 0 .5rem rgba(0,0,0,.1);
padding:clamp(2.2rem,6vw,4rem) clamp(1.3rem,5vw,3.4rem);animation:letter-in .5s ease-out both}
body.night .paper{box-shadow:0 0 .5rem rgba(0,0,0,.45)}
@keyframes letter-in{from{opacity:0}to{opacity:1}}
@media (prefers-reduced-motion:reduce){.paper{animation:none}}
.letter-date{margin:0 0 .5rem;font-size:.8rem;letter-spacing:.32em;color:var(--ink-soft)}
h1{margin:0 0 1.4rem;font-size:clamp(2.1rem,6vw,3rem);line-height:1.2;letter-spacing:-.02em;font-weight:600}
h2{margin:2.2em 0 .7em;font-size:1.28rem;line-height:1.35;letter-spacing:-.01em;font-weight:600}
h3{margin:1.9em 0 .5em;font-size:1.1rem;line-height:1.4;font-weight:600}
h4,h5,h6{margin:1.6em 0 .4em;font-size:1rem;line-height:1.5;font-weight:600}
.letter-body{overflow-wrap:break-word}
.letter-body p{margin:0;text-indent:2em;text-align:justify;
background-image:repeating-linear-gradient(transparent 0 calc(2em - 1px),var(--rule) calc(2em - 1px) 2em)}
.letter-body>p:first-of-type{text-indent:0}
.letter-body>p:first-of-type::first-letter{float:left;font-size:3em;line-height:1;font-weight:600;
color:var(--accent);padding:.16em .13em .84em 0}
blockquote{margin:1.6em 0;padding:.2em 0 .2em 1.2em;border-left:2px solid;
border-image:linear-gradient(to bottom,var(--accent),transparent) 1;color:var(--ink-soft)}
.letter-body blockquote p{text-indent:0;background-image:none;margin:.4em 0}
blockquote p:first-child{margin-top:0}
blockquote p:last-child{margin-bottom:0}
ul,ol{margin:.6em 0;padding-left:1.8em}
li{margin:.15em 0}
li::marker{color:var(--accent)}
.letter-body li p{text-indent:0;background-image:none;margin:0}
li input[type=checkbox]{accent-color:var(--accent);margin-right:.45em}
code{font-family:var(--mono);font-size:.85em;background:var(--code-bg);padding:.14em .45em;border-radius:4px}
pre{margin:1.4em 0;padding:1em 1.2em;background:var(--code-bg);border-radius:4px;overflow-x:auto;line-height:1.8}
pre code{background:none;padding:0;font-size:.88em}
a{color:inherit;text-decoration:underline;text-decoration-color:var(--accent);
text-decoration-thickness:1px;text-underline-offset:.28em}
a:hover{color:var(--accent)}
table{width:100%;border-collapse:collapse;margin:1.5em 0;font-size:.94em}
th,td{padding:.45em .7em;text-align:left;border-bottom:1px solid var(--hairline)}
th{font-weight:600;color:var(--ink-soft);letter-spacing:.05em}
del{color:var(--ink-faint)}
sup a{text-decoration:none}
.footnote-definition{margin:1em 0;font-size:.88em;color:var(--ink-soft)}
.letter-body .footnote-definition p{text-indent:0;background-image:none}
.divider{display:flex;align-items:center;gap:1.1rem;margin:2.6em 0;color:var(--accent)}
.divider::before,.divider::after{content:"";flex:1;height:1px}
.divider::before{background:linear-gradient(to left,var(--hairline) 30%,transparent)}
.divider::after{background:linear-gradient(to right,var(--hairline) 30%,transparent)}
.divider svg{display:block;width:112px;height:auto;opacity:.92}
hr.plain{border:0;height:1px;margin:2.6em 0;
background:linear-gradient(to right,transparent,var(--hairline),transparent)}
.letter-end{display:flex;justify-content:flex-end;margin-top:2.8em}
.sign{display:flex;flex-direction:column;align-items:center;gap:.7rem}
.sig-name{writing-mode:vertical-rl;letter-spacing:.3em}
.seal{width:1.55rem;height:1.55rem;display:grid;place-items:center;background:var(--accent);
color:#fdfbf6;border-radius:2px;font-size:.72rem;line-height:1;user-select:none}
.colophon{width:100%;max-width:38rem;margin:1.1rem auto 0;text-align:center;
font-size:.76rem;letter-spacing:.12em;color:var(--ink-faint)}
@media print{
body,body.night{--ink:#1a1a1a;--ink-soft:#444;--ink-faint:#666;--hairline:#bbb;
--code-bg:#f0f0f0;--edge:#ddd;background:#fff;padding:0;display:block}
.paper{max-width:none;border:0;border-radius:0;box-shadow:none;background:#fff;padding:0;animation:none}
.letter-body p{background-image:none}
.divider,.seal,.colophon{display:none}
a{text-decoration-color:currentColor}
}
"#;

/// frontmatter 元数据。
#[derive(Debug, Clone)]
pub struct Meta {
    pub title: String,
    pub date: String,
    pub signature: String,
    pub accent: String,
    pub decor: bool,
    pub footer: String,
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            title: "无题信".into(),
            date: String::new(),
            signature: String::new(),
            accent: "#e63946".into(),
            decor: true,
            footer: DEFAULT_FOOTER.into(),
        }
    }
}

/// 拆分 frontmatter 与正文。返回 (元数据, 正文 markdown)。
pub fn parse_source(src: &str) -> Result<(Meta, String), String> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    if !src.starts_with("---") {
        return Ok((Meta::default(), src.to_string()));
    }
    // 第一遍：先确认有收尾线，避免把正文行误报成字段错误。
    let mut offset = 0usize;
    let mut first = true;
    let mut body_start: Option<usize> = None;
    for line in src.split_inclusive('\n') {
        let bare = line.trim_end_matches(['\n', '\r']);
        if first {
            first = false;
            if bare.trim() != "---" {
                // 首行不是纯 "---"，当作没有 frontmatter 的正文。
                return Ok((Meta::default(), src.to_string()));
            }
            offset += line.len();
            continue;
        }
        let trimmed = bare.trim();
        offset += line.len();
        if trimmed == "---" || trimmed == "..." {
            body_start = Some(offset);
            break;
        }
    }
    let Some(body_off) = body_start else {
        return Err(
            "找到了 frontmatter 的开头，但没有等到收尾的 ---。\n在字段块之后补一行只写着 --- 的分隔线，再空一行接正文。"
                .to_string(),
        );
    };
    // 第二遍：严格解析字段。
    let mut meta = Meta::default();
    for line in src[..body_off].split_inclusive('\n').skip(1) {
        let trimmed = line.trim_end_matches(['\n', '\r']).trim();
        if trimmed == "---" || trimmed == "..." || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!(
                "frontmatter 里这一行没有冒号：「{trimmed}」。\n写成 `字段: 值` 的形式，比如 `title: 秋日一封信`。"
            ));
        };
        let value = unquote(value.trim());
        match key.trim() {
            "title" => meta.title = value,
            "date" => meta.date = value,
            "signature" => meta.signature = value,
            "accent" => meta.accent = normalize_accent(&value)?,
            "decor" => {
                meta.decor = value.parse::<bool>().map_err(|_| {
                    format!("decor 只接受 true 或 false，现在是「{value}」。")
                })?;
            }
            "footer" => meta.footer = value,
            _ => {} // 未识别的字段安静略过，不挡路。
        }
    }
    Ok((meta, src[body_off..].to_string()))
}

fn unquote(v: &str) -> String {
    let bytes = v.as_bytes();
    if bytes.len() >= 2
        && (bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"'
            || bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')
    {
        v[1..v.len() - 1].to_string()
    } else {
        v.to_string()
    }
}

/// 校验并归一化强调色：#RGB / #RGBA / #RRGGBB / #RRGGBBAA，统一小写带 #。
pub fn normalize_accent(input: &str) -> Result<String, String> {
    let hex = input.trim().strip_prefix('#').unwrap_or(input.trim());
    let ok_len = matches!(hex.len(), 3 | 4 | 6 | 8);
    if !ok_len || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "强调色「{input}」不是合法的 hex 颜色。\n写成 #RGB 或 #RRGGBB，例如：--accent #E63946。"
        ));
    }
    Ok(format!("#{}", hex.to_ascii_lowercase()))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Markdown → 单文件自包含 HTML。
pub fn render_html(meta: &Meta, body_md: &str, dark: bool) -> Result<String, String> {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let mut body_html = String::with_capacity(body_md.len() * 2);
    html::push_html(&mut body_html, Parser::new_ext(body_md, opts));
    let body_html = if meta.decor {
        body_html.replace("<hr />", DIVIDER_SVG)
    } else {
        body_html.replace("<hr />", "<hr class=\"plain\">")
    };

    let theme_color = if dark { NIGHT_THEME_COLOR } else { LIGHT_THEME_COLOR };
    let color_scheme = if dark { "dark" } else { "light" };
    let body_class = if dark { " class=\"night\"" } else { "" };

    let mut out = String::with_capacity(CSS.len() + body_html.len() + 1024);
    out.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n");
    out.push_str("<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str(&format!("<title>{}</title>\n", esc(&meta.title)));
    out.push_str(&format!(
        "<meta name=\"generator\" content=\"LitePen {}\">\n",
        env!("CARGO_PKG_VERSION")
    ));
    out.push_str(&format!("<meta name=\"theme-color\" content=\"{theme_color}\">\n"));
    out.push_str(&format!("<meta name=\"color-scheme\" content=\"{color_scheme}\">\n"));
    out.push_str("<style>");
    out.push_str(CSS);
    out.push_str("</style>\n</head>\n");
    out.push_str(&format!(
        "<body{body_class} style=\"--accent:{}\">\n",
        meta.accent
    ));
    out.push_str("<article class=\"paper letter\">\n");
    out.push_str("<header class=\"letter-head\">\n");
    if !meta.date.is_empty() {
        out.push_str(&format!("<p class=\"letter-date\">{}</p>\n", esc(&meta.date)));
    }
    out.push_str(&format!("<h1>{}</h1>\n</header>\n", esc(&meta.title)));
    out.push_str("<main class=\"letter-body\">\n");
    out.push_str(&body_html);
    out.push_str("\n</main>\n");
    if !meta.signature.is_empty() {
        out.push_str("<footer class=\"letter-end\">\n");
        out.push_str("<div class=\"sign\">");
        out.push_str(&format!(
            "<span class=\"sig-name\">{}</span>",
            esc(&meta.signature)
        ));
        out.push_str("<span class=\"seal\" aria-hidden=\"true\">笔</span>");
        out.push_str("</div>\n</footer>\n");
    }
    out.push_str("</article>\n");
    out.push_str(&format!("<p class=\"colophon\">{}</p>\n", esc(&meta.footer)));
    out.push_str("</body>\n</html>\n");
    Ok(out)
}

/// `litepen new` 的信笺模板。`today` 形如 2026-10-11。
pub fn template(today: &str) -> String {
    let footer = DEFAULT_FOOTER;
    format!(
        r##"---
title: 秋日一封信
date: {today}
signature: 某个人
accent: "#e63946"
decor: true
footer: {footer}
---

有人说秋天适合写信，日子变短，话变长。

这是 **LitePen** 的一张样笺。留下第一句，换成你想说的，信就成了。

## 这张纸上的规矩

- 首段第一个字会自动下沉，用强调色，像信的开头
- 一行 `---` 变成一枝花，放在意思转折的地方
- 强调色和页脚寄语写在 frontmatter 里就能换；`--dark` 给你一张夜笺

> 信很慢，但每一行都会到。

---

写完存成 `letter.md`，运行 `litepen build letter.md`，信就到了 `out/index.html`。用~~飞鸽~~浏览器打开，就能读。
"##
    )
}

/// 审计 HTML 里的外部资源引用（自包含检查）。
/// 内容里的 `<a href="https://…">` 是导航不是资源，不算。
pub fn audit_external(html: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    // 任何资源标签的出现都意味着要额外取文件，破坏自包含。
    for tag in [
        "<link", "<img", "<iframe", "<video", "<audio", "<source", "<track", "<embed", "<object",
    ] {
        collect_tag(html, tag, &mut found, 120);
    }
    // <script> 区分内联（自包含）与带 src 的外链。
    let mut from = 0usize;
    while let Some(rel) = html[from..].find("<script") {
        let start = from + rel;
        let snippet: String = html[start..].chars().take(160).take_while(|c| *c != '>').collect();
        if snippet.contains("src=") || snippet.contains("href=") {
            found.push(snippet);
        }
        from = start + "<script".len();
    }
    // CSS 里的 url(...) 与 @import（data: 之外都算外部）。
    let mut from = 0usize;
    while let Some(rel) = html[from..].find("url(") {
        let start = from + rel + 4;
        let inner: String = html[start..].chars().take_while(|c| *c != ')').collect();
        let t = inner.trim().trim_matches(['\'', '"']);
        if !t.is_empty() && !t.starts_with("data:") {
            found.push(format!("CSS url({t})"));
        }
        from = start;
    }
    collect_tag(html, "@import", &mut found, 80);
    found
}

fn collect_tag(html: &str, tag: &str, out: &mut Vec<String>, cap: usize) {
    let mut from = 0usize;
    while let Some(rel) = html[from..].find(tag) {
        let start = from + rel;
        let snippet: String = html[start..].chars().take(cap).take_while(|c| *c != '>').collect();
        out.push(snippet);
        from = start + tag.len();
    }
}

/// 人类可读的体积：B / KiB / MiB。
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut v = bytes as f64;
    let mut u = 0usize;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[u])
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn accent_normalization() {
        assert_eq!(normalize_accent("#E63946").unwrap(), "#e63946");
        assert_eq!(normalize_accent("3a6ea5").unwrap(), "#3a6ea5");
        assert_eq!(normalize_accent("#F60").unwrap(), "#f60");
        assert!(normalize_accent("red").is_err());
        assert!(normalize_accent("#12345").is_err());
    }

    #[test]
    fn frontmatter_splitting() {
        let src = "---\ntitle: T\ndecor: false\n---\n\n正文\n";
        let (meta, body) = parse_source(src).unwrap();
        assert_eq!(meta.title, "T");
        assert!(!meta.decor);
        assert_eq!(body, "\n正文\n");
    }

    #[test]
    fn no_frontmatter_falls_back() {
        let (meta, body) = parse_source("只有正文").unwrap();
        assert_eq!(meta.title, "无题信");
        assert_eq!(body, "只有正文");
    }

    #[test]
    fn unclosed_frontmatter_is_an_error() {
        assert!(parse_source("---\ntitle: T\n正文").is_err());
    }

    #[test]
    fn escape_in_meta() {
        let meta = Meta { title: "a\"<b".into(), ..Meta::default() };
        let html = render_html(&meta, "你好", false).unwrap();
        assert!(html.contains("<title>a&quot;&lt;b</title>"));
    }
}
