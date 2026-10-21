# 示例第一批与 `text_width`

- 日期：2026-10-20
- 状态：已实施
- 提交：`e930bd1` feat(text): export text_width；`919399e` feat(examples): port Ink examples without tests；`fbf07c2` feat(examples): port Ink examples covered by render tests
- Brief：[m4-p5a](briefs/m4-p5a.md)、[m4-p5a-fix1](briefs/m4-p5a-fix1.md)

## 背景

M4 的验收是逐个对照 Ink 的示例（见 031）。P1 合入后，不依赖 P2–P4 的示例可以与 P2 并行。cursor-ime 示例需要与渲染器一致的字符宽度，ink-rs 没有公开这样的函数。

## 决策

- 用户批准 M4 计划时（「可以」）同意：导出 `ink_rs::text_width(&str)`；示例的周期 tick 用 `use_animation`；jest、subprocess-output 走 tokio，放到后面的批次。
- Claude 在 brief 中定下（按 `research-examples-tokio.md` 的推荐，用户已批准）：
  - 布局：有测试的示例用 `examples/<name>/{main,app}.rs`，`app.rs` 只用公开 API，测试以 `#[path]` 引入；没有测试的示例是单文件。目录名保留 Ink 的 kebab-case，使 `cargo run --example use-input` 对应 Ink 的命令。
  - `text_width` 建在现有的 `styled::parse` 与 `measure` 上，不加新的宽度逻辑；约定为最宽一行的列数，ANSI 序列宽 0。
  - tick 的两种消费方式：按 tick 计数的状态由 `frame` 推出；其他状态在以 `frame` 为键的 `use_effect` 中每交付一帧走一步。每个示例在注释中写明用哪一种。`counter.rs` 保留线程加 `AppHandle` 的写法作对照。
  - 翻译规则：`useReducer` 改为 `State<T>` 加 `fn reduce`；`useRef` 改为 `use_memo` 的 `Rc<Cell<_>>`；同一次读入的多个按键用 `State::update`；`Math.random` 改为本地 xorshift，faker 改为固定数据，不加依赖。
  - aria、select-input 只移植视觉部分，aria 属性等 M5。incremental-rendering 去掉 `incrementalRendering` 选项，ink-rs 在所有模式下按格子 diff。
  - 第一批不放 PTY 用例：P2 要改 `tests/pty.rs`，PTY 用例与挂起示例一起放第二批，避免冲突。

## 方案

- 派给一个 Codex，与 P2（见 035）并行；brief 指明 P2 修改的文件不碰，`src/lib.rs` 只加 `text_width` 一行。
- 范围：20 个示例（alternate-screen、aria、border-backgrounds、borders、box-backgrounds、chat、cursor-ime、incremental-rendering、justify-content、render-throttle、scroll、select-input、static、table、use-animation、use-focus、use-focus-with-id、use-input、use-stderr、use-stdout）。
- 测试：`tests/examples.rs` 移植 18 个（`render.tsx` 16 个，`alternate-screen-example.tsx` 2 个），在进程内用 `testing::mount` 挂载，`waitForOutput` 改为越过节流窗口的 `advance`，字节检查改为屏幕行与光标位置。
- 验收命令：`cargo test --test examples --test text_width`、`cargo build --examples`、clippy。

## 结果

- 第一轮交付后 Claude review：逻辑无误；格式有问题。示例缺少条目之间的空行；多处长字符串让 rustfmt 放弃格式化整条链，`--check` 不报错，行宽超过 100。另发现 use-animation 示例在渲染中写 `Cell`，记录中未见处理。
- 退回 Codex 修（fix1）：长字面量提到 `const` 或 `let` 后再让 rustfmt 排版，补空行，测试中重复的行匹配提成 `has_row` 辅助函数。
- 集成后全量 1240 个测试通过，clippy 与示例构建无警告。按 Codex 建议拆成三个提交：`text_width`、无测试的 11 个示例、带测试的 9 个示例加测试与文档。
- 文档：`docs/ink-tests.md` 更新计数，加「示例的观测边界」条目；[design.md](../design.md#示例) 新增「示例」小节，宽度一条写入 `text_width` 的约定。
- 剩余的 terminal-resize、suspend-terminal（PTY）与 jest、subprocess-output（tokio）留给后续批次。
