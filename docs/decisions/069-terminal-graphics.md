# 终端图形原语：回复解析、`use_graphics` 探测与 `write_passthrough`

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：[terminal-graphics](briefs/terminal-graphics.md)

## 背景

用户问「ink-rs 和 eink-rs 不是支持图片吗 我们的markdown 能支持图片显示吗」。eink-rs 的 Markdown 组件把 `![alt](src)` 显示成 `[alt]`。eink-rs 的记录 020 调研了终端图片协议，022 调研了 GPUIX 的终端模式、herdr 和 tmux 的图形处理。结论：

- kitty 图形协议的 Unicode 占位符贴合 ink-rs 的帧模型。图像数据发一次；占位符是 `U+10EEEE` 加行列组合符的文字，前景色编码图像 id，随 diff 重写，随 `Static` 进入 scrollback。主 agent 验证过：`Text::new("\u{10EEEE}\u{305}\u{305}").color(Color::Rgb(0, 0, 42))` 写出的字节是 `\x1b[38;2;0;0;42m\u{10eeee}\u{305}\u{305}\x1b[39m`，不需要新元素。
- ink-rs 缺三样东西：
  - 输入解析器认的转义序列是 `ESC [` 和 `ESC O`。APC（`ESC _`）、DCS（`ESC P`）、OSC（`ESC ]`）的回复被拆成 `ESC` 加一个字符，剩下的字节成了按键。
  - 没有图形能力的探测和单元格像素尺寸。`src/driver/sys/unix.rs` 的 `size()` 丢掉了 `TIOCGWINSZ` 的像素字段。
  - 在 tmux 里，kitty 图形命令要经 `ESC P tmux; … ESC \` 透传到外层终端；不包裹的 APC 被 tmux 拿去设窗格标题（3.6b 的 `input_exit_apc`）。
- herdr（0.9.5）内嵌 libghostty，自己回答 kitty 查询、DA1、`CSI 16 t`，填写 `TIOCGWINSZ` 的像素字段，支持占位符，丢弃 `ESC P tmux;` 透传，并从 pane 环境里删掉 `TMUX`。tmux 3.6b 自己回答 DA1 和 `CSI 16 t`；外层终端对透传查询的回复能否回到程序，要实测。

主 agent 给出修订方案后，用户答「1. 这一版就做吧。 2. 可以的」，第二点确认方案。

## 决策

- 用户定：
  - 「1.直接改ink-rs」：这一版在 ink-rs 加原语，用 kitty 占位符。
  - 「1. 可以」（回复主 agent 的提议：第一次调用 `use_graphics` 时发出探测）：探测是惰性的。
  - 「ink-rs 不只看 tmux 也看看 herdr」：探测和透传覆盖直连、tmux 和 herdr。
  - 「不支持kitty的终端就先直接退化成alt。」：这一版的协议是 kitty，不做半块，ink-rs 不需要别的协议的探测。
  - 「2. 可以的」：确认下面三项原语。
- 三项原语（主 agent 提出，用户确认）：
  1. 输入解析器认出 APC、DCS、OSC 等字符串序列（以 ST 或 BEL 结尾），它们不作为按键送出，探测期间交给探测。
  2. `Scope::use_graphics()`：第一次调用时发出 kitty 查询 `a=q`、`CSI 16 t` 和 DA1，DA1 作哨兵，超时 1 秒。`TMUX` 有值时 kitty 查询和 DA1 经透传包裹，`CSI 16 t` 不包裹，由 tmux 回答。herdr 不需要专门的代码。结果是探测中、kitty、不支持三种，外加单元格的像素尺寸：先取 `TIOCGWINSZ`，为 0 时取 `CSI 16 t` 的回复。非交互模式和不能探测的终端得到不支持。
  3. `OutputWriter::write_passthrough(bytes)`：在 tmux 里包裹，其余情况和 `write_control` 相同。
- Claude 按用户的确认（「2. 可以的」）定细节：
  - 探测期间 ink-rs 打开 raw mode 并读输入，与输入 hook 的数量无关；结束后按输入 hook 的数量恢复。
  - `write_passthrough` 每次调用包裹成一条透传序列，ink-rs 不分块。分块是 kitty 协议的事（`m=1`），由 eink-rs 每块调用一次；tmux 主分支单条输入的上限 `input-buffer-size` 默认 1 MB，块是 4096 字节。方案原文写「包裹并分块」，这里改为调用方分块。
  - kitty 的回复是 `OK` 时算作支持。WezTerm 和 Konsole 回答 kitty 查询，但占位符不可用（ratatui-image 的兼容表；它的 `picker.rs` 在这两个终端里屏蔽 kitty），ink-rs 的帧模型里 kitty 要走占位符，所以 `TERM_PROGRAM=WezTerm` 或有 `KONSOLE_VERSION` 时不探测，得到不支持。
  - 探测期间把终端交出去（`suspend_terminal`、Ctrl+Z），探测按不支持结束。
  - 不在 Probing 时收到的字符串序列丢掉。DA1 和 `CSI 6;…t` 这类 CSI 回复在探测外按现有规则处理：Ink 丢掉没有对应按键的终端报告。

## 备选与取舍

- **启动时探测**：每个应用多一次往返，用不到图片的应用要付这个代价。用户选了惰性探测。
- **按环境变量判断支持与否**：在 tmux 和 herdr 里 `TERM_PROGRAM` 被改写，`KITTY_WINDOW_ID` 会过期或被删。环境变量的用途是决定要不要包裹（`TMUX`），排除已知不支持占位符的终端。
- **整屏位图（GPUIX 的做法）**：没有 scrollback，不能选择复制，要 GPU 栈。不适合文字界面。
- **直接放置加光标定位（ink-picture 的做法）**：和 ink-rs 的 diff、`Static` 冲突。
- **像素尺寸取自 `CSI 16 t`**：`TIOCGWINSZ` 不需要往返，resize 时跟着窗口尺寸更新，字号变化后宽高比保持准确。
- **字符串序列等到终结符为止**：Alt+P、Alt+_、Alt+] 发出的字节是 `ESC P`、`ESC _`、`ESC ]`。没有终结符时沿用 20 ms 的转义超时，超时后按现在的拆法送出，按键不受影响。

## 方案

一个 Codex worker（`codex exec -s workspace-write`）：

- 工作目录是 scratchpad 里的 worktree `ink-graphics`，基于 main；brief 见 [terminal-graphics](briefs/terminal-graphics.md)。
- 它不提交（沙箱写不了 `.git`），不改文档。它跑 fmt、两套 feature 的 clippy、`--lib` 和 brief 列出的集成测试。

主 agent 集成：

1. review，按三项原语分次提交。
2. fmt、两套 feature 的 clippy、全量测试。
3. `docs/design.md` 写探测、回复解析和透传。
4. 在 Ghostty 直连、tmux 会话 `ink-rs`、herdr 里手测：探测结果和耗时；tmux 里外层回复能否回到程序；`CSI 16 t` 的值；探测期间和之后屏幕上有没有回显的回复。
5. 补本记录的结果，删 worktree 和分支。发布 0.1.6 新开一条记录，等用户同意。eink-rs 的 `Image` 组件和 Markdown 接入在 eink-rs 的记录 023。

已知限制：herdr 里跑 tmux、tmux 里跑 herdr 这两种嵌套，程序识别不出，结果可能不对。
