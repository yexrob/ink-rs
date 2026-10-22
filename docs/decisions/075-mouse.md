# 鼠标：SGR 1006 解析、`use_mouse` 和命中测试

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：[mouse](briefs/mouse.md)；需求：[bingo-mouse](briefs/bingo-mouse.md)

## 背景

bingo 的全屏界面要点击、拖动、滚轮和悬停。今天解析器把 `ESC [ < b;x;y M` 当普通 CSI，`parse_keypress` 得不到名字后丢弃；读被切断时残段在 20 ms 后漏成按键。调研见 [071](071-bingo-research.md)，报告的结论：

- 内联模式下帧在屏幕上的行随 Static 输出、屏幕滚动、外部写入和缩放移动，ink-rs 不知道它，坐标规则无法成立。
- `use_box_metrics` 的位置相对父节点，命中测试要用 `measure_element` 的绝对坐标。
- 悬停需要 1003 模式。
- Windows Terminal 在 `ENABLE_VIRTUAL_TERMINAL_INPUT` 下把鼠标转成 VT 序列，现有读线程能收到。

## 决策

- 用户定：选「备用屏 + 悬停档 (Recommended)」：上报限于备用屏，做点击、拖动、悬停三档。
- Claude 按用户的选择定细节（调研报告的推荐）：
  - `use_mouse(Option<MouseTracking>, handler)`，`None` 停用；档位 `Click`（1000）、`Drag`（1002）、`Motion`（1003），按活跃 hook 的档位取高者。
  - 命中测试是 `measure_element(node).contains(x, y)`，库不按节点路由，不做点击聚焦。
  - 坐标以动态帧左上角为原点，和 `ElementMetrics` 同系；压在回放的 Static 行上时 `y` 为负。
  - 模式和粘贴模式同样处理：进恢复快照，挂起时关闭，恢复后下一步重开。
  - stdin 不是 TTY 时 hook 静默无效。
  - 不支持 X10 和 1015 编码；Windows 不改代码，文档写「Windows Terminal 支持」，旧版 conhost 未验证。

## 备选与取舍

- **内联模式用 `CSI 6n` 查询帧的位置**：结果限于查询的时刻，每次变动要一次异步往返，并和输入交错。
- **`use_mouse_in(node, ..)` 的回调限于矩形内的事件**：是全局 hook 的包装，拖出边界和悬停离开需要全局事件。
- **`active: bool` 加固定 1002**：悬停要 1003，需要档位参数。

## 方案

一个 Codex worker，worktree `wt/ink-mouse`。brief 见 [mouse](briefs/mouse.md)。测试：解析器的按下、释放、拖动、滚轮、修饰键，分片读取，超时丢弃残段；备用屏挂载下的点击坐标、Static 回放后的平移、模式字节（开启、换档、卸载和挂起关闭），内联和非 TTY 不写模式字节。验收：fmt；clippy 两组 feature 和 `x86_64-pc-windows-msvc`；`--lib`；brief 列出的输入和挂起测试。

主 agent 集成：review，提交，全量测试，`docs/design.md` 的输入和恢复两节，在 tmux `ink-rs` 会话里手动点击和悬停核对。赶上 0.1.7 时随它发布，赶不上放到 0.1.8。
