# ink-rs 设计

ink-rs 是 Rust 版的 [Ink](https://github.com/vadimdemedes/ink)：用组件和 flexbox 写终端界面。行为以 Ink 8.0.0（commit 26d2c3f）为准，Ink 的测试集是规格。

## 分层

上层依赖下层，下层不知道上层。

```
 API        builder：View Text Static Transform Newline Spacer、Styled、use_* hooks
 driver     sync │ tokio │ testing
              │ dispatch(event)、render_if_due(now) -> Option<Instant>
 ───────────────────── 核心（步进式，单线程）─────────────────────
 runtime    组件实例树、hooks、按 key 比对、调度、effect、context、焦点
              │ host 变更：创建 / 改属性 / 插入 / 删除
 host       自有节点树，实现 taffy 的底层 trait，跨帧常驻
 layout     taffy 布局  ←  text：解析、宽度、换行、测量
 paint      host 树 + 布局 → Frame
 output     Frame 差分 → 字节
 ──────────────────────────────────────────────────────────────
 Terminal   trait：crossterm 实现 │ 内存实现
```

括号里是 Ink 中对应的部分。

| 层 | 职责 | Ink |
|---|---|---|
| components / hooks | builder 和 `use_*` | `components/`、`hooks/` |
| driver | 拥有循环和睡眠，把事件交给核心 | `ink.tsx` 的事件接线 |
| runtime | 组件实例树、按调用顺序存放的 hooks、按 key 比对、调度、effect | `reconciler.ts` |
| host | 节点存储：View / Text / Static / Transform。自有节点树，实现 taffy 的底层布局 trait，跨帧常驻。Text 里嵌套的 Text、Transform、组件构成行内树，挂在 Text 节点上，taffy 看到的是一个叶子 | `dom.ts` |
| layout + text | taffy 布局；文本解析、测量、换行、截断、宽字符 | `styles.ts`、`wrap-text.ts` … |
| paint | 纯函数：节点树 → `Frame`（网格、新增的 Static 行、光标位置），含边框、背景、裁剪、Transform。每帧整帧重画 | `render-node-to-output.ts`、`output.ts` |
| output | 非全屏 / 全屏两种模式：单元格 diff、Static、光标、同步输出、恢复终端 | `log-update.ts`、`ink.tsx` |
| Terminal | 写字节、取尺寸、切换模式 | — |

项目起步是单个 crate，按层分模块：`text`、`host`、`layout`、`paint`、`runtime`、`app`、`output`、`input`、`driver`、`testing`。加过程宏时拆成 workspace。组件库（输入框、列表、spinner）放在单独的 crate，不进 v1。

### 跨层约定

- **单元格是内部表示。** text 层把字符串解析成带样式的字素（`StyledChar`），layout、paint、output 处理它。字符串出现在边界：`Text` 的内容、Transform 的参数和返回值、`render_to_string` 的结果、写给终端的字节。
- **时间由驱动传入。** 核心不读时钟，测试传假时间，不设 Clock 接口。
- **终端 I/O 经 `Terminal` trait。** 核心写字节、取尺寸、切换模式（raw、备用屏、粘贴、kitty 键盘）；读输入归驱动。crossterm 实现用于生产，内存实现放在 `testing` 模块。
- **终端恢复用 RAII。** 切换模式时返回 guard；drop、panic hook、SIGTSTP/SIGCONT 共用一份恢复逻辑。

## 编程模型

- **组件**是函数 `fn(&mut Scope, &P) -> Element`，要求 `P: PartialEq`。props 没变时跳过整棵子树，等于默认带 memo。
- **hooks** 按调用顺序存放。debug 构建记录每个位置的类型，顺序变化时 panic 并报出组件名。
- **状态句柄** `State<T>` 是 `Copy` 的分代句柄（runtime id、槽位、代数），读写经线程局部变量找到当前 runtime，放进闭包无需 clone。UI 运行时单线程、`!Send`，在其他线程使用句柄会 panic；其他线程通过 `Send` 的 `AppHandle` 提交更新。
- **写状态**：`set` 要求 `T: PartialEq`，值相等时不触发渲染；`update(|v| ..)` 表示明确修改，标脏。渲染过程中写状态在 debug 下 panic，派生值用 `use_memo`。
- **调度**：一轮事件处理完，所有更新合并成一帧，重渲染标脏的组件和 props 变化的子树。定时器和动画引起的更新受 `max_fps` 限制（Ink 默认 30），键盘输入引起的更新不受限制。
- **空闲**：没有变化时零唤醒、零渲染，由测试保证。
- **effect**：`use_layout_effect` 在布局之后、写到终端之前执行，可以读元素尺寸，写状态会让这一帧在写出前重算；`use_effect` 在这一帧写到终端之后执行。清理靠返回值的 `Drop`；组件卸载时它的异步任务随 drop 取消。

### 一帧

1. 驱动调 `dispatch`。按键交给活动的 `use_input` 回调，定时器到期和 `AppHandle` 的消息经同一入口进入。回调写状态，所属组件标脏。
2. `render_if_due(now)` 到期时：重跑脏组件（props 相等的子树跳过）→ 子元素按 key 比对，产出 host 变更 → taffy 重算 → paint 出 Frame → `use_layout_effect` → output 差分写出 → `use_effect`。
3. 返回下一次唤醒的时间；没有脏组件、没有定时器时返回 `None`。

## 事件循环

核心是步进式的，不拥有循环：

- `dispatch(event)` 处理一个事件，`render_if_due(now)` 在需要时渲染一帧，返回下一次需要唤醒的时间 `Option<Instant>`，`None` 表示空闲。驱动按这个时间睡眠，空闲零唤醒由它保证。
- 核心定义唤醒接口。其他线程经 `AppHandle` 提交更新，或 UI 线程上的任务写 `State`，两种情况通过它叫醒驱动。
- 调度规则（批量更新、`max_fps`、空闲零唤醒）写在核心里，与驱动无关。测试驱动手动推进循环和假时钟。

驱动有三个：

- **同步驱动**：UI 线程跑循环，一个线程阻塞读 crossterm 事件。异步结果经 `AppHandle` 送回 UI 线程。
- **tokio 驱动**（feature `tokio`，默认开启）：UI 循环跑在 `LocalSet` 上，`spawn_local` 派生的任务与 UI 同线程，可以写 `State`。任务句柄 drop 时 abort，组件卸载时任务随之取消。输入用 crossterm 的 `event-stream`。`use_future` 属于这个 feature。
- **测试驱动**（`testing` 模块）：内存终端，手动推进，传假时间。

testing 驱动在 M1，同步驱动在 M2，tokio 驱动在 M4。CI 跑一遍 `--no-default-features`，覆盖同步驱动。

不以 tokio 为核心的理由：从步进式核心加 tokio 写法是多写一个驱动，从 tokio 核心退回运行时无关要改公开 API；调度顺序由自己的循环决定，不交给 `select!`；测试可以按确定的顺序推进循环。

## 写法

`Box` 与 `std::boxed::Box` 重名，改叫 `View`（同 React Native）。v1 的写法是 builder：rustfmt 能格式化，rust-analyzer 能补全，编译器能检查类型。宏 DSL 放到 M5，作为 builder 的语法糖，能力不超出 builder。

```rust
fn counter(cx: &mut Scope, p: &CounterProps) -> Element {
    let count = cx.use_state(|| 0);
    let step = p.step;
    cx.use_input(move |_, key| {
        if key.up { count.update(|c| *c += step) }
    });
    View::new()
        .padding(1)
        .border_style(BorderStyle::Round)
        .child(Text::new(format!("count: {}", count.get())).color(Color::Green))
        .into()
}
```

M0 的 API（`src/lib.rs`）：

- `Element`：`View`、`Text`、`Static`、`Transform`、`Newline`、`Spacer`、`Option<T>`、`Vec<T>` 可以转成它；`Element::fragment` 对应 `<>…</>`。字符串不是 `Element`，与 Ink 一样，文本要放进 `Text`，`View::child("x")` 编译不过。
- `Styled` trait：方法名是 Ink 的 Box props 改 snake_case。数字是格子数，`Percent(50.0)` 是百分比，颜色用 `Color` 枚举，`"…".parse::<Color>()` 解析 Ink 的颜色字符串。
- `Text` 的子节点是 `TextChild`：字符串、嵌套的 `Text`/`Transform`、`Newline`，或渲染成文本的 `Element`。
- `render_to_string(element, RenderOptions { columns, color_level })`：Static 输出在前，行尾没有多余空格（带样式的格子除外），末尾没有换行。

包名 `ink-rs`，lib 名 `ink_rs`。crates.io 上的 `ink` 属于 Parity 的智能合约框架 ink!，对外介绍时注明两者无关。

## 布局与文本

- **布局**用 taffy 的 flexbox 算法，开放 Ink 用到的属性。host 树实现 taffy 的底层 trait（不用 `TaffyTree`），跨帧保留，每帧重算变化的节点。与 Yoga 的差异分两处抹平。样式映射：根节点用 Yoga 的默认值；没设最小尺寸的节点最小尺寸为 0（Yoga 没有 min-content 下限）；文本宽度向上取整；`position: static` 映射成 relative 并忽略 inset。布局时由 host 树调整子节点的测量约束和样式：自动尺寸的子项以容器的可用尺寸为上限测量 flex basis（Yoga 的 AtMost，taffy 按 max-content）；设了 aspect ratio 的子项由主轴尺寸推出交叉轴尺寸，不看交叉轴的尺寸和 stretch。有子项按 baseline 对齐时，Yoga 按内容决定行高，这一点不复刻，记在 `ink-tests.md`。
- **ANSI**：用户传入的字符串在 `Text` 边界解析一次。保留 SGR 和 OSC 8 超链接，去掉其他控制序列与 C0 控制字符（制表符和换行除外），防止外部内容移动光标。超链接以外的 OSC（窗口标题、剪贴板等）在解析时丢掉。超链接记下原始参数和终止符（BEL 或 ST），输出时还原。
- **样式叠加**：`Text` 的样式按"内层没设的属性由外层补上"叠加到字素上，不复刻 chalk 的字符串包裹。props 里的颜色按 `color_level` 降级（同 chalk）；用户传入的原始 SGR 不降级。
- **Transform**：把一行序列化成字符串交给用户函数，解析返回值。
- **断行**同 wrap-ansi：在空格处断行，单词超过行宽时按字素硬断，不去掉首尾空格。不按 UAX #14：一段连续的中日韩文字放得进下一行时，wrap-ansi 把整段移到下一行，UAX #14 在当前行行尾断开，Ink 的测试期望是前者。截断同 cli-truncate，按行进行：首尾截断时省略号取保留一侧的样式，不带超链接；中间截断和只剩 1 列时省略号不带样式。换行与截断模式同 Ink：`Wrap`、`Hard`、`TruncateStart`、`TruncateMiddle`、`TruncateEnd`。
- **宽度**按字素簇（unicode-segmentation）计算，规则同 string-width 8.3：零宽字素不占格，emoji 序列和东亚宽字符占 2 格，东亚歧义宽度按 1 格。
- **测量缓存**存在 Text 节点上，文本或可用宽度变化时失效。不做 Ink 按字符串的全局缓存：那是每帧重新拼字符串的结果，常驻树用不到。

## 终端输出

- **diff 是唯一有状态的部分。** paint 是纯函数，输入节点树和终端尺寸，输出网格，每帧整帧重画；局部重画等有了性能数据再讨论。diff 保存上一帧网格，保证它与屏幕上的内容一致。
- **按单元格比较。** 样式在变化处切换。行尾用 EL（清到行尾）代替补空格：补出的空格在窗口变窄时会被终端重排成乱行。
- **同步输出。** 每帧包在 `CSI ?2026` 里，避免闪烁。
- **非全屏模式的规则：不写之后够不着的行。** `Static` 写进 scrollback 一次，此后不改。底部可刷新区域的高度上限是屏高，超出部分从顶部裁掉，不清屏、不整屏重绘。持续增长的内容应放进 `Static`。
- **resize** 没有精确解法，各家终端重排行为不同，策略在真实终端上测过之后定。
- **光标**对应 Ink 的 `useCursor`：组件声明光标位置，每帧结束后把真实光标移过去，输入法候选框靠它定位。
- **非 TTY / CI**：不刷新画面，输出 Static 内容和退出时的最后一帧。
- **恢复终端**：退出、panic、Ctrl+Z 挂起时恢复终端模式；SIGCONT 后重画。

## 输入

crossterm 负责读事件和切换终端模式，不负责画面输出。hooks 与 Ink 对应：`use_input`、`use_paste`、`use_focus` / `use_focus_manager`、`use_app`、`use_window_size`、`use_box_metrics`、`use_animation`（所有动画共用一个定时器）。存在活跃的 `use_input` 时进入 raw mode，没有时不进入。

## 测试

Ink 的测试是规格，分四层：

1. `render_to_string`：纯函数，M0 的验收。
2. `testing::render`：对应 ink-testing-library，能取最后一帧、模拟按键、替换时钟（M1）。
3. 输出字节交给 vt100 模拟器执行，检查屏幕和 scrollback（M2）。
4. portable-pty 跑示例程序，测 raw mode、Ctrl+C、退出码、挂起（M3 起）。

各家终端的差异模拟器测不出，要在真实终端上手动验收。

### 移植 Ink 测试的方法

- **期望值来自 Ink 自己。** 在 Ink 的测试运行中记录每条断言的实际值和期望值（测试通过，二者相等），转成 Rust 表达式。不手算 boxen、chalk 生成的期望。
- **样式用标记写。** `tests/common` 的 `styled(&[...])` 把 `{fg=green,bold}x{/}` 这类标记展开成 SGR；`assert_ansi_eq` 按"每个字符的样式 + 结尾残留样式"比较，不锁定 SGR 的拆分和顺序。Ink 用 `includes()` 检查 SGR 字节的测试，改为比较记录下来的完整输出。
- **渲染参数同 Ink 的测试 helper**：`render` 是 100 列、16 色（`FORCE_COLOR=true`）；设了 `chalk.level = 3` 的测试用 `render_truecolor`；调用库函数 `renderToString` 的测试默认 80 列。
- **一一对应。** 一个 Ink 测试文件对应 `tests/` 下一个 Rust 文件（内部函数的测试放在模块的 `tests.rs`），文件头注明来源和 commit。测试名、顺序与 Ink 一致。
- **内部函数的测试调用内部 API。** `tests.rs` 里写字符串与内部表示之间的转换，期望值不变，不为测试保留字符串签名。
- **Ink 实现的副作用不模拟。** 有的期望值来自 JS 实现的副作用，例如稀疏数组的空洞、重复叠加的转义序列，照做要写只为测试服务的代码。这类测试改期望，原因记进 [ink-tests.md](ink-tests.md#与-ink-测试的差异)。
- **每个里程碑移植它能表达的测试。** 依赖运行时、终端或输入的测试，随对应里程碑移植。每个 Ink 文件的状态见 [ink-tests.md](ink-tests.md)。
- Ink 上游 skip 的测试移植后标 `#[ignore]`，写明原因。

## 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| M0 | View/Text、布局、文本、边框、`render_to_string`；元素是数据，没有运行时 | 移植的 Ink 布局和文本测试通过 |
| M1 | 组件树、hooks、按 key 比对、调度、测试驱动与 `testing::render` | 断言重渲染次数，空闲时零渲染 |
| M2 | 两种输出模式、Static 跨帧、光标、非 TTY、终端恢复、同步驱动 | vt100 场景测试 |
| M3 | 输入、焦点、粘贴、kitty 键盘协议、退出 | PTY 测试 |
| M4 | 动画、元素尺寸测量、panic 时的错误界面、挂起恢复、tokio 驱动；移植 Ink 的示例 | 逐个对照示例 |
| M5 | 宏 DSL、屏幕阅读器模式 | — |

## 平台

v1 支持 Linux 和 macOS，不支持 Windows。终端模式、SIGCONT、挂起这类 Unix 专属代码放在 `cfg(unix)` 后面，不写 Windows 分支，以后支持 Windows 时改动集中在这几处。
