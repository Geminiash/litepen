# 轻笔 · LitePen 设计说明：排版系统拆解

> 视觉依据：工作区 `docs/visual-guide.md` 第 9 节（轻笔信笺）与第 11 节（三项目视觉规格速查表）。
> 所有装饰均为纯 CSS / 内联 SVG，零位图，零运行时 JS。

## 1. 色板令牌

亮色（纸）与暗色（夜笺）各一组 CSS 变量，全页只消费令牌：

| 令牌 | 纸（默认） | 夜笺（`--dark`） | 用途 |
|---|---|---|---|
| `--paper-1` / `--paper-2` | `#faf8f2` / `#f5f1e8` | `#1c1c32` / `#131324` | 纸张纵向渐变两端 |
| `--backdrop` | `#eae3d4` | `#0b0b16` | 页面背景（纸外） |
| `--ink` | `#3a352e` 墨色 | `#e8d4a6` 暖金 | 正文 |
| `--ink-soft` | `#6f6759` | `#c8b287` | 日期、引言、脚注 |
| `--ink-faint` | `rgba(58,53,46,.45)` | `rgba(232,212,166,.45)` | 页脚寄语 |
| `--hairline` | `rgba(58,53,46,.24)` | `rgba(232,212,166,.26)` | 分隔线两侧细线、表格线 |
| `--rule` | `rgba(58,53,46,.055)` | `rgba(232,212,166,.05)` | 行间横线（与行高 2em 同步） |
| `--accent` | `#e63946`（Stripe 红，落款章色） | 同用户色 | 首字下沉、分隔线花枝、列表符号、链接下划线、印章 |

- 强调色经由 `<body style="--accent:…">` 注入，CSS 本体不掺值；CLI 与 frontmatter 的值都先过 `normalize_accent`（`#RGB/#RGBA/#RRGGBB/#RRGGBBAA`，统一小写），非法值直接报错而不是静默回退。
- 对比度：墨色对纸底约 11.3:1，暖金对深靛约 12.6:1，均远超 4.5:1 底线；强调红只落在大字号（首字下沉）与装饰上。

## 2. 字体与字号

- 衬线栈（visual-guide 第 7 节）：`"Noto Serif SC", "Songti SC", "Source Han Serif SC", SimSun, serif`；等宽 `"SF Mono", Consolas, Menlo, monospace`。
- 标题小字距宽行高（Apple 式负字距）：`h1` `clamp(2.1rem, 6vw, 3rem)`、`letter-spacing -.02em`、行高 1.2、字重 600；`h2` 1.28rem、`-.01em`、1.35。
- 正文 1rem、**行高 2**（两主题一致的中文规范）；段落 `text-indent: 2em` + `text-align: justify`，首段除外（让位给首字下沉）。
- 日期行：0.8rem + `letter-spacing .32em`，宽字距小字是信头的呼吸感。

## 3. 首字下沉（三行）

```css
.letter-body > p:first-of-type::first-letter{
  float:left; font-size:3em; line-height:1; font-weight:600;
  color:var(--accent); padding:.16em .13em .84em 0;
}
```

- 关键在 padding 的换算：`::first-letter` 内的 `em` 以放大后的字号（3em = 48px）为基准。上补白 `.16em ≈ 7.7px`，让字的顶端对齐首行文字顶（行高 2 的半行距 = 8px）；下补白 `.84em ≈ 40px`，使浮动盒总高 = 7.7 + 48 + 40 ≈ **96px = 3 × 2em 行框**，恰好让三行文字绕排。
- 只作用 `first-of-type` 的直接段落：正文以标题开篇时，下沉自动让给第一个真正的段落。
- 3em 字号与 visual-guide 第 9 节样例（3.2em）同量级，视觉上是"信的开头"，不是海报大字。

## 4. 花枝分隔线（`---`）

- 渲染时把 pulldown-cmark 输出的 `<hr />` 替换为内联 SVG（约 1KB）：五瓣勾线小花 + 两根卷须花茎 + 叶与蕾，`stroke="currentColor"`，随容器色（`var(--accent)`）变色。
- 两侧渐隐细线沿用 shoka divider 手法：`::before` / `::after` 各占 `flex:1`，`linear-gradient` 从透明渐到 `--hairline`。
- `decor: false` 时 `---` 退为一条居中渐隐的素线（`hr.plain`），落款印章随之隐藏；`@media print` 亦然。

## 5. 纸张质感与行间横线

- 纸张 = 顶部暖光 + 纵向纸色渐变（guide 第 9 节原文）：
  `radial-gradient(ellipse 80% 40% at 50% -10%, rgba(255,220,180,.10), transparent)` 叠 `linear-gradient(var(--paper-1), var(--paper-2))`。
- 行间横线只画在正文段落上：`repeating-linear-gradient(transparent 0 calc(2em - 1px), var(--rule) calc(2em - 1px) 2em)`——横线间距与行高 2em **严格同步**。段落 `margin: 0`，相邻段落的横线因此连续，像真正的信纸。
- 引言、列表、脚注里的段落显式关掉横线与缩进（`.letter-body blockquote p` 等选择器压过 `.letter-body p` 的特异性）。
- 纸片圆角 4px（速查表"内敛，纸片感"），阴影 `0 0 .5rem rgba(0,0,0,.1)`（夜笺加深至 .45）；不用玻璃拟态。

## 6. 引言、落款与页脚

- 引言：`border-left: 2px solid` + `border-image: linear-gradient(to bottom, var(--accent), transparent) 1`——左侧一条从强调色渐隐的细线；文字用 `--ink-soft`，不用斜体（中文斜体是假斜）。
- 落款：`writing-mode: vertical-rl` 竖排签名 + `--accent` 底色的小方章（"笔"字），flex 右对齐，像信末的钤印。
- 页脚寄语：纸外一行居中小字（`--ink-faint`），默认"手写于深夜与清晨之间，感谢读到这一行。"，frontmatter `footer` 可覆盖。

## 7. 夜笺、打印与动效

- 夜笺：`body.night` 换令牌（深靛纸 `#1c1c32 → #131324`、暖金墨 `#e8d4a6`），`<meta name="color-scheme" content="dark">` 与 `theme-color` 同步翻转。
- 打印：`@media print` 重新定义一组令牌（白纸黑字、灰发丝线），隐藏分隔线 / 印章 / 页脚寄语，正文与落款全保留。
- 动效：仅入场 `fade-in .5s`（速查表允许的仅有的两段动画之一），且 `prefers-reduced-motion: reduce` 时直接关闭；`::selection` 用强调色，是唯一的隐藏彩蛋。

## 8. 自包含与 `--check`

- 产物单文件：CSS 内联、装饰为内联 SVG、无 `<link>` / `<script>` / `<img>` / `url()`。
- `--check` 扫描资源型标签（`link/img/iframe/video/audio/source/track/embed/object`）、带 `src` 的 `<script>`、CSS `url()`（`data:` 除外）与 `@import`；内容导航 `<a href="https://…">` 不算资源。逐条列出上下文，便于定位。
- `serve` 是 std 手写的 TcpListener + 每连接一线程：正确 MIME 表、样式化 404/405、percent-decode 后拒绝 `..` 与反斜杠、canonicalize 后校验根目录包含关系，双保险防穿越。
