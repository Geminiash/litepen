# 轻笔 · LitePen

> 一份 Markdown 进，一封像信的 HTML 出。
> One Markdown in, one letter-like HTML out.

轻笔是一个单二进制 CLI：把一份 Markdown 渲染成一封"像信一样好看"的静态页面——邀请函、信笺、独立小页。产物是**单个自包含的 index.html**：CSS 全内联、无外链、无图片依赖、零运行时 JS，直接发送或托管都可以。

视觉语言遵循工作区视觉规范（`docs/visual-guide.md` 第 9、11 节）：暖米白纸底 + 墨色正文，衬线字体栈，首字下沉，`---` 化作一枝花，竖排落款加一枚红章；`--dark` 一键换深靛夜笺、暖金墨色。

## 特性

- **首字下沉**：首段第一个字放大三行，强调色，像信的开头
- **花枝分隔线**：Markdown 的 `---` 渲染为内联 SVG 花枝，两侧渐隐细线，随强调色变色
- **纸感排版**：行高 2 的中文正文，行间隐约横线与之严格同步；标题负字距宽行高；引言块左侧渐变细线
- **竖排落款**：signature 竖排 + 强调色印章
- **夜笺**：`--dark` 生成暗色变体（深靛纸底 + 暖金墨色）
- **强调色**：`--accent #RRGGBB` 或 frontmatter `accent` 字段，一处处换全页
- **打印友好**：`@media print` 去装饰留正文
- **`--check`**：报告正文体积，审计是否含外部资源引用
- **serve**：std 手写的极简静态服务器，本地预览即开即用

依赖只有 2 个 crate：`clap`（derive）与 `pulldown-cmark`。

## 快速开始（中文）

```bash
# 构建
cargo build --release

# 生成一张信笺模板
./target/release/litepen new letter.md

# 填好想说的话，渲染
./target/release/litepen build letter.md             # 输出 out/index.html
./target/release/litepen build letter.md -o site/    # 指定输出目录
./target/release/litepen build letter.md --dark      # 夜笺变体
./target/release/litepen build letter.md --accent "#2E5E4E" --check

# 本地预览（默认 http://127.0.0.1:4141）
./target/release/litepen serve out/
```

### frontmatter 字段

```markdown
---
title: 秋日一封信          # 信的标题（默认"无题信"）
date: 2026-10-11           # 日期行，留空则不显示（`new` 生成 UTC 当天）
signature: 某个人           # 竖排落款，留空则不显示落款块
accent: "#e63946"          # 强调色（落款章色，默认 #E63946）
decor: true                # 装饰开关：花枝分隔线等；false 时 --- 变素线
footer: 手写于深夜与清晨之间，感谢读到这一行。   # 页脚寄语
---

正文从这里开始。首段第一个字会自动下沉。
```

### CLI 一览

| 命令 | 说明 |
|---|---|
| `litepen new <file.md>` | 生成带 frontmatter 的信笺模板（已存在则拒绝覆盖） |
| `litepen build <file.md>` | 渲染为单个自包含 `index.html`，默认输出 `out/` |
| `litepen build -o <dir>` | 输出到指定目录（目录不存在会创建） |
| `litepen build -o <file.html>` | 直接写到该文件 |
| `litepen build --dark` | 暗色夜笺变体 |
| `litepen build --accent <hex>` | 覆盖强调色（`#RGB` / `#RRGGBB`） |
| `litepen build --check` | 报告体积 + 审计外部资源引用 |
| `litepen serve <dir> [-p 4141]` | 极简静态服务器预览目录（`-p 0` 随机端口，绑定 127.0.0.1） |

## English Quick Start

```bash
cargo build --release
./target/release/litepen new letter.md
./target/release/litepen build letter.md --check
./target/release/litepen serve out/
```

A complete example letter (`examples/letter.md`):

```markdown
---
title: 秋日一封信
date: 2026-10-11
signature: 某个人
accent: "#e63946"
decor: true
footer: 手写于深夜与清晨之间，感谢读到这一行。
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
```

Frontmatter fields: `title`, `date`, `signature`, `accent`, `decor`, `footer` — all optional, all overridable. The output is one self-contained HTML file: inlined CSS, no external references, no JavaScript. `--check` prints the size and audits external resource references (content links like `<a href>` don't count; `<img>`, `<link>`, `<script src>` and CSS `url()` do).

## 构建与体积

| 产物 | 体积 |
|---|---|
| `litepen` release 二进制（Windows x64，`opt-level="z"` + LTO + strip） | 626,176 字节（约 611 KiB） |
| 示例信笺产物 `index.html`（亮色） | 7,775 字节（约 7.6 KiB） |
| 示例信笺产物 `index.html`（夜笺） | 7,788 字节（约 7.6 KiB） |

> 在 Windows + Git Bash 下，若 `link.exe` 解析到 coreutils 的 `link`，请为构建指定 MSVC 链接器：
> `export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="<VS 安装路径>/VC/Tools/MSVC/<版本>/bin/Hostx64/x64/link.exe"`，
> 并按需补 `LIB`（MSVC `lib/x64`、SDK `um/x64` 与 `ucrt/x64`）。

## 测试与 CI

- `cargo test`：5 个单元测试（frontmatter 拆分 / accent 归一化 / HTML 转义）+ 11 个集成测试（全字段渲染、markdown 元素转换、`--check` 输出、serve 的 MIME/404/防穿越/405）
- CI：GitHub Actions，stable 与 1.85 两个工具链跑 `cargo test --locked` 与 release 构建
- 排版自检流程：build 示例信笺 → playwright 在 360/1280 两档截图核对首字下沉、分隔线、暗色变体（截图用后即删，不入库）

## 设计说明

排版系统的完整拆解（色板令牌、字号行高、首字下沉的实现、花枝分隔线、夜笺配色、打印样式、动效降级）见 [docs/design.md](docs/design.md)。

## License

MIT
