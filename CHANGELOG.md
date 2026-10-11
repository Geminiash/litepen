# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-11

### Added

- `litepen new <file.md>`：生成带 frontmatter（title / date / signature / accent / decor / footer）的信笺模板；已存在的文件拒绝覆盖。
- `litepen build <file.md> [-o out/]`：渲染单个自包含 `index.html`（CSS 全内联、无外链、零 JS）。
  - 排版：纸感浅色主题、衬线正文栈、首段首字下沉三行（强调色）、`---` 花枝分隔线（内联 SVG 随强调色）、行高 2 的中文正文与同步行间横线、标题负字距宽行高、引言块左侧渐变细线、竖排落款 + 强调色印章。
  - `--dark`：暗色夜笺变体（深靛纸底 + 暖金墨色）。
  - `--accent <hex>`：覆盖强调色（`#RGB` / `#RRGGBB`，归一化校验）。
  - `@media print` 去装饰留正文；`prefers-reduced-motion` 降级。
- `litepen build --check`：报告正文体积并审计外部资源引用。
- `litepen serve <dir> [-p 4141]`：std 手写极简静态服务器（正确 MIME、404/405 页、路径穿越防护、`-p 0` 随机端口）。
- 测试：5 单元 + 11 集成（frontmatter 全字段、markdown 元素转换、`--check`、serve MIME/404/防穿越）。
- CI：stable 与 1.85 两个工具链。

[0.1.0]: https://example.com/litepen/releases/tag/v0.1.0
