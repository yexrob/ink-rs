# M3 调研与输入架构
- 日期：2026-10-20
- 状态：已实施
- 提交：`8bdef1f` docs(design): record the M3 input architecture
- Brief：派给 Claude 子 agent，prompt 未存档

## 背景
M3 覆盖输入（`use_input`）、粘贴（`use_paste`）、kitty 键盘协议、焦点（`use_focus`、`use_focus_manager`）和退出。按约定，里程碑调研完成后先与用户定方案，再派活。

11:54 派出三个 general-purpose 子 agent：盘点 M3 输入测试、盘点焦点与退出测试、总结 Ink 输入语义。结论：
- 输入类测试 283 个，可移植 270 个；焦点与退出类 94 个，可移植 77 个（进程内 64 个，PTY 13 个），另列出待定问题 Q1–Q9。
- 用 crossterm 解析时，52 项对照中 18 项复现不了：crossterm 每个字符发一个事件，没有 Esc 的 20 ms 等待，吞掉 kitty 查询回复，不接受内存字节。Ink 的 92 个解析器单元测试一个也移植不了；自己移植解析器后 89 个可原样移植。

## 决策
用户在提问中选定：
- 输入：「移植 Ink 解析器 (Recommended)」。驱动读 stdin 字节，核心解析，新增 `filedescriptor 0.8`。
- raw mode：「rustix termios，去掉 crossterm (Recommended)」。照 libuv 的 raw 分支保留 OPOST、设置 ONLCR，与 Node 一致；尺寸改用 rustix `tcgetwinsize`。Claude 用 deepwiki 核对过 libuv 的标志位。
- 退出值：用户先问「你推荐用哪个 为什么」，Claude 推荐 `Box<dyn Any + Send>`，理由是对应 Ink 的任意值，而泛型方案中 `use_app` 拿不到 `T`。用户回复：「那个泛型还有Any的就按照你说的用Any」。
- 字段名：「enter / super_key (Recommended)」。

Claude 自行决定，并告知用户可以推翻：
- 公开类型命名为 `KeyEvent`，因为 `Key` 已用于元素 key。
- 持有 raw mode 的 hook 让应用保持存活，对应 Ink 的 `stdin.ref()`，由 E13 锁定。
- 外部卸载用 `AppHandle::exit()`，因为等待期间 `Instance` 已被消耗。
- Tab、Esc、Ctrl+C 的派发放在核心 Runtime，两种测试驱动都能用。
- 焦点在 layout effect 中注册；无焦点时 Shift+Tab 照 Ink 代码跳到最后一项（Ink readme 写的是第一项）；禁用状态下 `focus(id)` 照 Ink 设置 active；注销重复 id 时只删本 hook 的条目。
- `Uint8Array` 输入的 3 个测试不移植。
- stdin EOF 时读线程停止；多应用时 raw mode 计数与原始 termios 按进程保存，只开一个读线程，输入交给最近注册、持有 raw mode 的应用。

## 备选与取舍
- crossterm 事件：代码少，但 18 项行为复现不了，同步读取停不下来。
- 保留 crossterm raw mode：`cfmakeraw` 清掉 OPOST，要保留 `\n`→`\r\n` 改写；原始 termios 存在全局静态里，多应用要另外计数。
- 泛型 `Instance<T>`：组件里要写 `use_app::<T>()`，类型不匹配只能在运行时 panic。不带值：E7、E8 要改写。
- `return_key` 或 `r#return`：前者与 Ink 对应，后者难看。

## 方案
- 解析：移植 `input-parser.ts` 与 `parse-keypress.ts`，纯函数。
- 读取：读线程用 `filedescriptor::poll` 等待 stdin 和 socketpair 唤醒端；raw mode 计数归零即停止读取。
- 核心入口 `App::input(bytes, now)`：派发顺序为 kitty 回复、粘贴、Ctrl+C、Esc/Tab 焦点、按注册顺序的 `use_input`；一次调用合并为一次提交；Esc 20 ms 与 kitty 200 ms 并入 `render_if_due` 的截止时间。
- SIGWINCH 仍由 signal-hook 线程转发。`Terminal` 增加 `stdin_is_tty` 与 `set_raw_mode`。
- 分期：P1 解析器与 P2 退出 API 并行；P3 派发与 hooks；P4 焦点与 P5 生产驱动并行。

## 结果
- 13:10 提交 `8bdef1f`，design.md 增加 [输入](../design.md#输入) 各条和新的 [退出](../design.md#退出) 一节。
- 各期见 [024](024-m3-input-parser.md)、[025](025-m3-exit-api.md)、[026](026-m3-use-input.md)、[027](027-m3-paste-kitty.md)、[028](028-m3-focus.md)、[029](029-m3-production-driver.md)。P3 拆成 P3a 与 P3b。
