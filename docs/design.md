# ink-rs 设计

ink-rs 是 Rust 版的 [Ink](https://github.com/vadimdemedes/ink)：用组件和 flexbox 写终端界面。行为以 Ink 8.0.0（commit 26d2c3f）为准，Ink 的测试集是规格。

## 分层

上层依赖下层，下层不知道上层。

```
 API        builder：View Text Static Transform Newline Spacer、Styled、use_* hooks
 driver     sync │ tokio │ testing
              │ dispatch(event)、render_if_due(now) -> Option<Instant>
 ───────────────────── 核心（步进式，单线程）─────────────────────
 app        节流窗口、待绘制状态、终端写入与恢复
 runtime    组件实例树、hooks、按 key 比对、effect、context、焦点
              │ host 变更：创建 / 改属性 / 插入 / 删除
 host       自有节点树，实现 taffy 的底层 trait，跨帧常驻
 layout     taffy 布局  ←  text：解析、宽度、换行、测量
 paint      host 树 + 布局 → Frame
 output     Frame 差分 → 字节
 ──────────────────────────────────────────────────────────────
 Terminal   trait：进程实现 │ 内存实现
```

括号里是 Ink 中对应的部分。

| 层 | 职责 | Ink |
|---|---|---|
| components / hooks | builder 和 `use_*` | `components/`、`hooks/` |
| driver | 拥有循环和睡眠，把事件交给核心 | `ink.tsx` 的事件接线 |
| app | 接收驱动解析的模式值，管理节流、输出模式、终端写入与恢复 | `ink.tsx` 的渲染与卸载 |
| runtime | 组件实例树、按调用顺序存放的 hooks、按 key 比对、调度、effect | `reconciler.ts` |
| host | 节点存储：View / Text / Static / Transform。自有节点树，实现 taffy 的底层布局 trait，跨帧常驻。Text 里嵌套的 Text、Transform、组件构成行内树，挂在 Text 节点上，taffy 看到的是一个叶子 | `dom.ts` |
| layout + text | taffy 布局；文本解析、测量、换行、截断、宽字符 | `styles.ts`、`wrap-text.ts` … |
| paint | 纯函数：节点树 → `Frame`（网格、新增的 Static 行），含边框、背景、裁剪、Transform。每帧整帧重画；光标意图保存在 runtime 的提交状态，由 app 随帧交给 output | `render-node-to-output.ts`、`output.ts` |
| output | 单元格 diff、Static、光标、擦除与结束帧字节 | `log-update.ts`、`ink.tsx` |
| Terminal | 写字节、取尺寸、切换模式 | — |

项目起步是单个 crate，按层分模块：`text`、`host`、`layout`、`paint`、`runtime`、`app`、`output`、`input`、`driver`、`testing`。过程宏放在 workspace 成员 `macros`（`ink-rs-macros`），由默认开启的 `macros` feature 引入。组件库（输入框、列表、spinner）放在单独的 crate，不进 v1。

### 跨层约定

- **单元格是内部表示。** text 层把字符串解析成带样式的字素（`StyledChar`），layout、paint、output 处理它。字符串出现在边界：`Text` 的内容、Transform 的参数和返回值、`render_to_string` 的结果、写给终端的字节。
- **时间由驱动传入。** 核心不读时钟，测试传假时间，不设 Clock 接口。
- **终端 I/O 经 `Terminal` trait。** 核心写字节、取尺寸、切换模式（raw、备用屏、粘贴、kitty 键盘）；读 stdin 归驱动，解析与分发归核心。进程实现（`driver::process`）用于生产，内存实现放在 `testing` 模块。
- **终端恢复用 RAII。** app 的 drop、panic hook、终止信号共用 Terminal 的恢复操作与已恢复标记；驱动保存跨线程可达的恢复快照，核心不依赖驱动的 hook。核心安装自己的 panic hook，记录会被捕获的 panic，见[退出](#退出)。挂起与 SIGTSTP/SIGCONT 见[终端输出](#终端输出)。

## 编程模型

- **组件**是函数 `fn(&mut Scope, &P) -> Element`，要求 `P: PartialEq`。props 没变时跳过整棵子树，等于默认带 memo。`component(f, props)` 生成组件元素，组件身份按 `f` 的类型区分。
- **子元素和回调作 props**：`Element` 实现 `PartialEq`，按结构比较到组件边界，组件节点比较类型和 props，Transform 与 Static 的函数按指针比较。回调用 `Callback<A, R>`，按指针比较。`use_callback(f)` 每次渲染返回同一个句柄，调用时执行最近一次渲染传入的闭包；没有依赖数组，同 Dioxus 的 `use_callback`，不同于 React 的 `useCallback`。
- **比对**同 React：子元素有 key 按 key 对应，没有 key 按位置对应；类型或 key 变了，卸载旧实例、挂载新实例。
- **hooks** 按调用顺序存放。debug 构建记录每个位置的类型，顺序变化时 panic 并报出组件名。
- **状态句柄** `State<T>` 对任意 T 都是 `Copy + Send + Sync` 的分代句柄（runtime id、槽位、代数、所属线程 id），读写经线程局部变量找到当前 runtime，放进闭包无需 clone。UI 运行时单线程、`!Send`，在其他线程使用句柄会 panic；其他线程通过 `Clone + Send + Sync` 的 `AppHandle::update(FnOnce() + Send + 'static)` 提交更新。`Scope::use_app()` 与 `Instance::handle()` 返回句柄，core 不保留计数句柄；保留句柄表示有未结束的工作。核心的 FIFO 队列在下一步执行闭包，状态写入与事件回调相同；卸载关闭队列并丢弃闭包，后续 update 也丢弃。`OutputWriter` 同为 `Copy + Send + Sync`，每次写入检查所属线程，离开 UI 线程时 panic。一个线程可以同时运行多个实例。组件卸载后，写它的状态被忽略。
- **写状态**：`set` 要求 `T: PartialEq`，值相等时不触发渲染；`update(|v| ..)` 表示明确修改，标脏。渲染过程中写状态在 debug 下 panic，派生值用 `use_memo`。
- **调度**：一轮事件处理完，所有更新合并成一次提交：重渲染标脏的组件和 props 变化的子树、比对、布局、执行 `use_layout_effect`。提交不节流；paint 和写出受 `max_fps` 限制（默认 30），同 Ink：窗口内第一次立即写出，其余合并到窗口结束时写出。窗口是 `1s / max_fps`，不取整；Ink 取 `ceil(1000 / max_fps)` 毫秒，默认 34ms。帧按固定节拍排：一帧比截止时间晚不到半个窗口，下一个截止时间从原截止时间算，系统晚醒不降低平均帧率；晚半个窗口以上，从这一帧重新起格。Ink 从实际写出的时刻算，晚醒累积（见[记录 063](decisions/063-frame-grid.md)）。Static 有新条目、resize、卸载、外部写、`clear()` 不等窗口。
- **空闲**：没有变化时零唤醒、零渲染，由测试保证。
- **effect**：`use_layout_effect` 在布局之后、paint 之前执行，可以读元素尺寸，写状态会让这一帧在写出前重算，一帧写出一次。Ink 在提交时写出，之后执行 layout effect，状态变了写第二帧；ink-rs 不写这个中间帧。`use_effect` 在提交之后执行，这一帧没被节流时在写出之后，同 React：passive effect 跟着提交走，不等节流后的写出。清理靠返回值的 `Drop`；组件卸载时它的异步任务随 drop 取消。
- **context**：`provide(value)` 把值交给组件返回的子树，`use_context::<T>()` 返回最近祖先提供的值，没有时为 `None`。按类型区分，同类型的两个 context 用 newtype。组件读不到自己 provide 的值，同 React 中 Provider 位于返回的树里。两个都是 hook，占调用顺序的位置。实例的祖先和 hook 顺序在它的生命周期内不变，所以 reader 挂载时找到 provider，用 `Dirty` 的弱引用登记。provider 的值变化时标脏 reader，标记向上冒泡，reconcile 穿过 props 相等、被跳过的中间组件，在同一次提交重渲染 reader，不出中间帧；值相等时不触发。
- **继承的属性**：View 的背景色经内建 context 传给后代 Text，同 Ink 的 `BackgroundContext`。背景色变化时，运行时更新下面受影响的 Text 节点，props 相等的组件不重跑：Ink 中重渲染的是读这个 context 的 Text，不是用户组件。
- **元素尺寸**：`use_node_ref` 给出句柄，挂到 `View::node_ref`。`measure_element` 返回 x、y（相对根节点）、width、height，以及去掉边框的 client 尺寸，节点未挂载时全为 0。在 effect 和事件回调里读，读到的是最近一次布局。`use_box_metrics(node_ref)` 返回 width、height、left、top（相对父节点）、client 尺寸和 `has_measured`，布局变化时重渲染调用它的组件。ref 的槽存 `Option<布局>`，`None` 表示未挂载。读者登记在 Store，提交的每一轮刷新 ref 后比较读者上次返回的度量，变了标脏，这一帧写出前重算；布局不变不标脏，空闲时没有提交，也不检查。

### 一帧

1. 驱动调 `dispatch`。按键交给活动的 `use_input` 回调，定时器到期和 `AppHandle` 的消息经同一入口进入。回调写状态，所属组件标脏。
2. `render_if_due(now)`：有脏组件时提交：重跑脏组件（props 相等的子树跳过）→ 子元素按 key 比对，产出 host 变更 → taffy 重算 → `use_layout_effect`，写了状态回到第一步。节流窗口允许时 paint 出 Frame → output 差分写出。最后执行 `use_effect`。
3. 返回下一次唤醒的时间：节流窗口的结束或最近的定时器；没有脏组件、没有待写出的帧、没有定时器时返回 `None`。

## 事件循环

核心是步进式的，不拥有循环：

- `dispatch(event)` 处理一个事件，`render_if_due(now)` 在需要时渲染一帧，返回下一次需要唤醒的时间 `Option<Instant>`，`None` 表示空闲。驱动按这个时间睡眠，空闲零唤醒由它保证。
- 核心定义唤醒接口。其他线程经 `AppHandle` 提交更新，或 UI 线程上的任务写 `State`，两种情况通过它叫醒驱动。
- 调度规则（批量更新、`max_fps`、空闲零唤醒）写在核心里，与驱动无关。测试驱动手动推进循环和假时钟。
- **动画**：所有 `use_animation` 共用 Store 中的一个调度表，同 Ink App 的共享定时器。订阅从最近一步的时间开始，核心不读时钟。每步开头 tick：到期的条目按 Ink 的追赶公式 `start + (floor(elapsed / interval) + 1) * interval` 前移，迟到的唤醒跳过错过的格点，`frame` 随之跳变；条目记一个格点锚，未到「锚 + 窗口」的条目不写状态，到期的 tick 保持到期，窗口一开就交付；Ink 丢掉这个 tick，等下一个格点，interval 略短于窗口时（60 fps 配 16ms）只剩一半。交付时取这次 tick 的 interval 格点，比门槛晚不到半个窗口时锚前移一个窗口，否则锚取这个格点。格点不含晚醒，晚醒不降低帧率，interval 大于窗口的动画也不会因锚漂移丢 tick。Ink 的门槛是距上次交付一个窗口。截止时间是各条目 `max(next_due, 锚 + 窗口)` 的最小值，节流窗口内的唤醒不发生。tick 经 `State::set` 写值。暂停、改 interval、`reset` 释放订阅，恢复时在 layout effect 中归零并重新订阅，归零值在同一帧写出。`testing::render` 与 `render_to_string` 用 0 窗口。
- app 管理节流窗口，runtime 不保存节流窗口：动画的 tick 与截止时间按 app 每步传入的窗口计算。Static 有新条目或身份改变时，提交后的绘制满足待绘制帧，但不打开或延长窗口。`max_fps = 0` 使用 0 窗口，同 Ink 的非正值分支。

驱动有三个：

- **同步驱动**：`render(element, Options) -> Instance` 挂载并运行首步后返回；交互首帧在返回前写出，非交互 live 帧仍在卸载时写出。`rerender`、`clear` 后各运行一步；`unmount(self)` 与 Drop 卸载。`wait_until_exit(self) -> Result<Option<Box<dyn Any + Send>>, Error>` 在调用线程排空更新队列、处理消息、用真实时钟步进，并阻塞等待消息或截止时间，结果见“退出”。`Instance` 为 `!Send`。平台事件源通知尺寸变化（Unix 用 SIGWINCH，Windows 轮询）；raw mode 期间读线程送来 stdin 字节，见“输入”。异步结果经 `AppHandle` 送回 UI 线程。
- **tokio 驱动**（feature `tokio`，默认开启）：`ink_rs::tokio::{render, Instance}`，方法同同步版，`wait_until_exit` 为 async。`render` 与 `wait_until_exit` 须在 `LocalSet` 或 local runtime 中运行；丢弃 `wait_until_exit` 的 future 即卸载。两个驱动共用挂载函数、`Instance` 的公共部分和每轮循环的 `Looper::turn`（输入合并、消息处理、步进、退出、挂起、自然结束），只在等待方式与挂起回调的运行位置上不同。消息经 `Post` 接口进入各自的通道：同步驱动用 std mpsc，tokio 驱动用 tokio unbounded channel，后者的 send 同步、不需要 runtime，读线程、信号线程与持队列锁的唤醒都能调用。tokio app 的所有时刻取自 tokio 时钟（`tokio::time::Instant::now().into_std()`），暂停的 tokio 时钟驱动 app；等待用 `timeout_at(deadline, recv())`。挂起回调经 `spawn_blocking` 运行，交出终端期间同线程的任务照常运行，它们的 `State` 写入提交但不绘制；回调 panic 时在 UI 线程 `resume_unwind`，不恢复，同同步驱动。终止信号卸载后 await `pending()` 而不 park 线程，让同线程其他 app 完成清理，最后一个注册移除时终止进程。同线程的多个 tokio app 并发，不算嵌套：tokio app 进入循环时不标记其他 app；同步 wait 阻塞整个线程，进入时把同线程所有在循环中的 app 标为循环外，结束时放回。输入沿用读线程，经核心的唤醒接口叫醒 UI 循环；不用 `AsyncFd`，它要求在 stdin 的打开文件描述上设 `O_NONBLOCK`，这个标志与 shell 和其他读者共享。
- **`use_future(deps, f)`**（feature `tokio`）：建在 `use_effect` 上，effect 中 `spawn_local(f())`，返回的清理在 drop 时 abort 任务：deps 变化与卸载都取消旧任务。effect 中取一个 `AppHandle` 放进任务，任务运行期间应用不自然结束；裸 `spawn_local` 不持有它，需要时自己捕获 `use_app()`。任务在 UI 线程、所有 runtime 捕获区之外被 poll：进程 panic hook 会恢复终端，tokio 又吞掉 panic，留下终端已恢复的 live app。所以任务的 future 包一层，每次 poll 在核心捕获区内进行，panic 写入 `Store` 上的首个 panic 槽（与 runtime 捕获区共用，槽从 Runtime 移到 Store，任务只持 Store 的弱引用），再唤醒循环；下一步按组件 panic 处理，显示错误界面，以 `Err(Error::Panic)` 退出。Store 已释放时丢弃。没有 `LocalSet` 时 `spawn_local` panic。`testing::mount` 在 `LocalSet` 中可用，任务随测试的 await 推进。
- **测试驱动**（`testing` 模块）：`mount(element, Options, &TestTerminal)` 挂载并运行首步；`advance` 从当前时间开始运行截止时间内的步骤。测试默认 CI=false、Basic 颜色，与 Ink 的测试 helper 一致。`testing::render` 保留不节流的帧记录模式，不添加运行时 debug 选项；它的 `input` 与 `advance` 用自己的测试时钟送入输入、交出等待中的 Esc，没有 raw mode 检查；`settle` 不推进时钟，停在未来的截止时间前，`advance` 逐个运行目标时间内的截止时间，每个 tick 有自己的 `delta`。`TestApp::live_frame()` 和 `static_output()` 读渲染器的帧，不解析字节：`testing::mount` 给 `AppOutput` 装一个记录器，`paint` 在备用屏合并 Static 之前记下 live 行，Static 行按写入顺序累积，`Static` 实例更换时不清空。行的序列化同 `TestInstance`，以 `\n` 连接。生产路径没有记录器。读不到光标位置、结构化的颜色、`use_stdout`/`use_stderr` 的外部写入和 `write_control` 的字节。

同步循环的自然结束条件集中在一处：步进返回 `None`，且没有外部 `AppHandle`、队列为空，对应 Ink `waitUntilExit` 的 Node `beforeExit`。无句柄的静态应用在首帧后结束；组件 render 中创建后丢弃的句柄不延长生命周期。raw mode 持有者不为 0 时不结束，对应 Node 的 `stdin.ref()`；stdin 读到 EOF 后不再算，之后有持有者重新开启 raw mode 时恢复；活动的动画给出唤醒时间，应用不结束，同 Ink 持有引用的定时器；暂停或卸载后不再计入。队列以 mutex 保护提交顺序和句柄计数；提交、最后句柄 drop、UI 状态写入通过核心 `Wake` 接口发消息。两个驱动的通道都保留检查与睡眠之间的消息，不丢唤醒，不定期轮询。`TestApp` 在 rerender / advance 的步骤执行队列，`testing::render` 在 settle 的步骤执行；`render_to_string` 不执行渲染期间提交的更新，卸载丢弃它们。

生产 Terminal 用 `IsTerminal` 查询 stdin 与 stdout，用 rustix `tcgetwinsize` 查询 stdout 的尺寸，失败返回 None，同 Node 的 `stdout.columns`；同一调用的像素字段除以列数和行数，得到单元格的像素尺寸，为 0 时未知。复制 fd 1/2 为 `File`，每个操作调用一次 `write_all` 后 flush，不经过 Rust stdout 的行缓冲。保留首个 I/O 错误并丢弃后续写入；循环在步骤边界结束、卸载并返回该错误，包括 Rust 忽略 SIGPIPE 后的 EPIPE。fd 复制或信号设施初始化失败也经此结果返回，未开启终端模式。

testing 驱动在 M1，同步驱动在 M2，tokio 驱动在 M4。CI 跑一遍 `--no-default-features`，覆盖同步驱动。

不以 tokio 为核心的理由：从步进式核心加 tokio 写法是多写一个驱动，从 tokio 核心退回运行时无关要改公开 API；调度顺序由自己的循环决定，不交给 `select!`；测试可以按确定的顺序推进循环。

## 写法

`Box` 与 `std::boxed::Box` 重名，改叫 `View`（同 React Native）。v1 的写法是 builder：rustfmt 能格式化，rust-analyzer 能补全，编译器能检查类型。宏 DSL 是 `rsx!`，作为 builder 的语法糖，能力不超出 builder：`Name(位置参数, 方法: 值) { 子节点 }` 展开为 `Name::new(..)` 或 `Name::default()`，路径带泛型参数时写成 `<Name>::…`，再按顺序调用方法，子节点交给 `child`，`key` 最后调用。没有裸标志，写 `bold: true`。子节点可以是元素、字符串、`{表达式}`、`if`、`for`；含 `{` 的字符串展开为 `format!`。`if` 与 `for` 在语句里串接父 builder，各分支不必同类型。多个根节点合成 `Element::fragment`。

组件在 rsx 里和 `View` 同样写法，靠 props 结构体上的 `#[derive(Props)]`：每个字段生成同名 setter，收字段类型，标 `#[props(into)]` 的收 `impl Into<T>`（整数字面量过不了 `impl Into<u16>`，所以不作默认）；`children: Vec<Element>` 字段生成 `child` 和 `children`；另生成 `key`，以及调用 `#[props(component = f)]` 所指组件函数的 `From<props> for Element`。结构体是数据，组件函数是渲染。派生不生成 `Default` 和 `new`，必填 props 由用户写 `new`，在 rsx 里作位置参数；不检查必填项，不做默认值属性。

```rust
#[derive(Default, PartialEq, Props)]
#[props(component = layout)]
struct Layout {
    #[props(into)]
    title: String,
    children: Vec<Element>,
}

fn layout(_: &mut Scope, p: &Layout) -> Element {
    rsx! {
        View(flex_direction: FlexDirection::Column) {
            Text(bold: true) { {p.title.clone()} }
            {p.children.clone()}
        }
    }
    .into()
}

rsx! { Layout(title: "Deploy") { Text { "step 1" } } }
```

```rust
fn counter(cx: &mut Scope, p: &CounterProps) -> Element {
    let count = cx.use_state(|| 0);
    let step = p.step;
    cx.use_input(true, move |_, key| {
        if key.up_arrow { count.update(|c| *c += step) }
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
- `render_to_string(element, RenderOptions { columns, color_level })`：Static 输出在前，行尾没有多余空格（带样式的格子除外），末尾没有换行。M1 起经运行时：挂载、渲染一帧、取输出、执行 `use_effect`、卸载。`use_layout_effect` 写的状态进入输出，`use_effect` 写的不进入，同 Ink。

包名 `ink-rs`，lib 名 `ink_rs`。crates.io 上的 `ink` 属于 Parity 的智能合约框架 ink!，对外介绍时注明两者无关。

## 布局与文本

- **布局**用 taffy 的 flexbox 算法，开放 Ink 用到的属性。host 树实现 taffy 的底层 trait（不用 `TaffyTree`），跨帧保留，每帧重算变化的节点。与 Yoga 的差异分两处抹平。样式映射：根节点用 Yoga 的默认值；没设最小尺寸的节点最小尺寸为 0（Yoga 没有 min-content 下限）；文本宽度向上取整；`position: static` 映射成 relative 并忽略 inset。布局时由 host 树调整子节点的测量约束和样式：自动尺寸的子项以容器的可用尺寸为上限测量 flex basis（Yoga 的 AtMost，taffy 按 max-content）；设了 aspect ratio 的子项由主轴尺寸推出交叉轴尺寸，不看交叉轴的尺寸和 stretch。后者经子项的样式视图实现：交叉轴尺寸改为 auto，确定的交叉尺寸按比例换到主轴，stretch 改为 flex-start。taffy 在伸缩前定下交叉尺寸，主轴经 grow / shrink 改变后不按比例重算，这一点与 Yoga 不同。有子项按 baseline 对齐时，Yoga 按内容决定行高，这一点不复刻，记在 `ink-tests.md`。
- **ANSI**：用户传入的字符串在 `Text` 边界解析一次。保留 SGR 和 OSC 8 超链接，去掉其他控制序列与 C0 控制字符（制表符和换行除外），防止外部内容移动光标。超链接以外的 OSC（窗口标题、剪贴板等）在解析时丢掉。超链接记下原始参数和终止符（BEL 或 ST），输出时还原。
- **样式叠加**：`Text` 的样式按"内层没设的属性由外层补上"叠加到字素上，不复刻 chalk 的字符串包裹。props 里的颜色按 `color_level` 降级（同 chalk）；用户传入的原始 SGR 不降级。`Color::Reset` 表示终端默认前景色或背景色，不添加 SGR；用于 `Text::background_color` 时阻断 `View` 的背景继承，对应 Ink 的 `backgroundColor=""`，嵌套 Text 保留外层 Text 的样式。
- **Transform**：把一行序列化成字符串交给用户函数，解析返回值。`Text` 里嵌套的 Transform 在文本节点 squash 时调用：文本 props 变化后，第一次测量或绘制时调用一次。Ink 在每次测量和渲染时调用，函数没有副作用时两者输出相同。外层 Transform 在每次绘制时调用，同 Ink。
- **断行**同 wrap-ansi：在空格处断行，单词超过行宽时按字素硬断，不去掉首尾空格。`Wrap` 和 `Hard` 对进入换行处理的各行做 NFC 归一化，同 wrap-ansi；放得下的文本和截断模式不做归一化。不按 UAX #14：一段连续的中日韩文字放得进下一行时，wrap-ansi 把整段移到下一行，UAX #14 在当前行行尾断开，Ink 的测试期望是前者。截断同 cli-truncate，按行进行：首尾截断时省略号取保留一侧的样式，不带超链接；中间截断和只剩 1 列时省略号不带样式。换行与截断模式同 Ink：`Wrap`、`Hard`、`TruncateStart`、`TruncateMiddle`、`TruncateEnd`。
- **宽度**按字素簇（unicode-segmentation）计算，规则同 string-width 8.3：零宽字素不占格，emoji 序列和东亚宽字符占 2 格，东亚歧义宽度按 1 格。属性表由 `scripts/gen_width_data.py` 从 Unicode 18.0.0 数据生成，这是 get-east-asian-width 1.7.0 使用的版本；东亚宽度遵循 get-east-asian-width，W/F 占 2 格，其余占 1 格。Ink 的其他属性来自 Node 的 ICU，因此在旧版 Node 上，Ink 与 ink-rs 对 Unicode 18.0 新字符的处理可能不同。公开的 `text_width(&str)` 返回渲染给最宽一行的列数：ANSI 序列不占格，制表符按 8 列展开。`use_cursor` 的列要与渲染一致，`unicode-width` 在 ZWJ emoji 与谚文字母上给出不同的宽度。
- **测量缓存**有两层。文本节点在 props 变化后第一次测量或绘制时 squash 行内树、展开制表符，结果和自然宽高存在 host 节点上，到下一次 props 变化为止；测量和绘制读它，按宽度断行在用到时做。布局用各节点的 taffy 布局缓存。节点变更时清掉它和祖先的缓存，同 Yoga 的 `markDirty`。不做 Ink 按字符串的全局缓存：那是每帧重新拼字符串的结果，常驻树用不到。

## 终端输出

- **diff 是唯一有状态的部分。** paint 是纯函数，输入节点树和终端尺寸，输出网格，每帧整帧重画；局部重画等有了性能数据再讨论。diff 保存上一帧网格，保证它与屏幕上的内容一致。
- **按单元格比较。** 逐行比较，变化的行从第一个变化格写到最后一个变化格，起止对齐宽字符边界，样式在变化处切换。行尾不补空格：补出的空格在窗口变窄时会被终端重排成乱行；新内容比旧内容短时用 EL（清到行尾）擦掉余下的格子。写到最后一列之后不写 EL：xterm 在待换行状态下执行 EL 会擦掉最后一格。Ink 默认擦掉整帧重写，增量模式按行从第一个变化列重写到行尾。
- **光标移动用相对序列。** 行间用 CUU、CUD、CR，新增的行写 `\r\n`，不依赖终端的 `ONLCR` 把 `\n` 转成 `\r\n`。
- **向下擦除**用 EL 擦掉当前行，右移一格后发 ED，最后 CR。tmux 默认开启 `scroll-on-clear`：ED 从屏幕原点开始时，先把整屏推进 scrollback 再清屏。帧顶位于屏幕第一行时，`CR CSI J` 会在 scrollback 里留下整帧副本。一列宽的屏幕无法右移，不在处理范围。
- **同步输出。** app 包装 Output 产出的字节。帧、`clear()`、主屏恢复各为一次操作，每次操作一次 write，`CSI ?2026h` 和 `CSI ?2026l` 在同一个缓冲里；无字节时不写同步序列；适用范围为交互式 TTY。卸载前的待绘制帧与恢复是两个操作。备用屏退出不包装同步序列，以 `CSI ?1049l`、`CSI ?25h` 结束，之后不写字节。Ink 把同步序列与帧分成三个 write。
- **Static 是 host 节点。** 每个实例记住已写出的条目数，长度缩短时降低计数；交给 output 的 `Frame::static_lines` 是本帧新增行，output 不保留 Static 历史。不复刻 Ink 的 index 状态和每次追加的两次提交。输出选择树中第一个 Static，同 Ink；其身份改变时 Frame 通知消费者重置历史。写出及本次提交的 passive effect 结束后，各 Static 卸载本次条目，执行清理，不发起第二次提交；后续 paint 前经过布局。新条目写在帧上方：回到帧顶，写新条目，再写帧，条目随终端滚动进 scrollback。`Static::from_fn(len, render)` 按下标渲染应用自己存的条目，历史增长时不复制；`Static::new` 经它构造。Ink 没有这个入口：JS 的数组按引用传，`Rc<[T]>` 追加一项要复制整个切片（见[记录 066](decisions/066-static-from-fn.md)）。
- **非全屏模式的规则：不写之后够不着的行。** `Static` 写进 scrollback 一次，此后不改。底部可刷新区域的高度上限是屏高，超出部分从顶部裁掉，不清屏、不整屏重绘。持续增长的内容应放进 `Static`。Ink 不设上限：超高帧整帧写出，下一帧用 `CSI 1;1H CSI J` 清掉可见区重写，每个超高帧把顶部的行往 scrollback 推一遍。两者的可见画面相同，ink-rs 的 scrollback 里没有重复的行。卸载时最后一帧超过屏高：擦掉可见部分，不设上限重写整帧，被裁掉的行进入 scrollback 一次，退出后 scrollback 里有完整的最后一帧，同 Ink。`clear` 之后卸载不重写。信号和 panic hook 的紧急恢复写裁剪后的快照，不重写整帧。
- **光标停留的位置**同 Ink：帧比屏幕矮时停在帧下一行的行首；帧高达到屏高时停在最后一行，不写末尾的换行。退出时光标移到帧下方，帧高达到屏高时补一个 `\r\n`，让 shell 提示符从新行开始（Ink 不补）。
- **全屏模式**即备用屏，启动时由选项决定，运行中不切换。帧从屏幕第一行开始。Static 没有 scrollback 可去，同 Ink 保留全部条目，Static 身份改变时清空历史。每帧取历史末尾至多一个屏高的行，放在 live 行之前，交给 Output 裁剪、差分，不整屏清除，不复制全部历史。
- **resize** 没有精确解法，各家终端重排行为不同。尺寸变化来自平台事件源：Unix 的 signal-hook iterator 线程转发 SIGWINCH，Windows 的 ink-resize 线程每 100ms 查询尺寸。UI 步骤查询 Terminal 尺寸；读线程在 raw mode 期间读 stdin，resize 不经过它。未知或为 0 的维度按 80 列、24 行回退。交互模式在宽度变小时按上一帧重排后的行数擦除：每个 live 行是独立逻辑行，按 `Cell::Glyph` 与其 `Cell::Continuation` 的宽度遍历字形；当前物理行有内容且余格放不下字形时换行，空行占一行，宽于新列数的字形占一行。光标取上一绘制帧的实际位置（停留位置或声明位置），映射到逻辑行内承载该格的物理行；光标在内容之后时取最后一个物理行，同 tmux 实测，短帧下方空行的偏移为 0。上移距离为光标逻辑行之前各行重排高度之和，加上光标在本逻辑行内的物理行偏移；隐藏光标，上移后向下擦除，忘记上一帧。宽度未变小而高度变小的规则照 Ink `resized`：上一绘制帧声明了光标时，从该光标回到帧顶向下擦除并忘记上一帧，避免碰到帧上方的历史；否则，上一帧高度达到新行数时，用 `CSI 1;1H` 回到屏幕原点，向下擦除可见区并忘记上一帧；其余情况走 diff。高度变小时，Output 累加终端推入 scrollback 的帧行数；高度增长时，上移越过这些行后向下擦除，因为 tmux 和多数终端会把它们拉回屏幕。增长量不足时保留 scrollback 中剩余行的计数；帧内 diff 保留计数，Static、外部输出、退出和挂起恢复清零。宽度收缩的计数条件是帧顶至光标的各行未发生换行重排；宽度小于已记录帧行的最大宽度，或 live 行重排使历史来源改变时清零。备用屏无计数。Ink 无此处理。重布局与绘制不等节流窗口，满足待绘制帧，不打开或延长窗口；擦除与绘制合用一次同步输出，帧未变且仍放得下时无写入。非交互模式更新尺寸供后续布局使用，不由 resize 触发擦除或绘制。runtime 改根宽度并安排布局，将调用 `Scope::use_window_size()` 的组件标脏；hook 在 render 读取 `WindowSize { columns, rows }`，不依赖 passive effect 订阅。`testing::render` 与 `render_to_string` 使用 `RenderOptions::columns` 和 24 行。宽度收缩规则偏离 Ink 的 `log.clear`：Ink 按绘制行数擦除，在 tmux 等重排终端留下旧帧顶部；ink-rs 按重排行数擦除，代价是在 xterm 等不重排的终端多擦除重排所增加的历史行。发生换行重排的帧超出屏高时，上移停在屏幕第一行，进入 scrollback 的部分无法擦除，Ink 有相同限制。tmux 以屏幕底部为锚重排：光标下方的空行保留，重排增加的行把屏幕顶部同样数量的行推进 scrollback；帧靠近屏幕顶部时帧头进入 scrollback，可见屏幕干净，之后加宽时这些行回到屏幕。tmux 按行内写过的最远格计算行长，EL 不缩短这一长度；diff 用 EL 截短的行在 tmux 中按旧长度重排，宽度收缩时可能少擦。Ink 增量模式的测试要求截短的行不重写未变的前缀，ink-rs 不改为整行重写。备用屏不重排，但帧从第一行开始，多出的上移停在第一行，不设分支。
- **光标**对应 Ink 的 `useCursor`：组件在 render 调用 `Scope::use_cursor(Option<CursorPosition>)`，`CursorPosition { x: usize, y: usize }` 是从 0 开始的整数格，原点为 Static 下方 live 帧的左上角；`None` 隐藏。备用屏把 Static 历史放在 live 前交给 Output 时，app 给 y 加上本次重放的 Static 行数；Output 按高度上限减去裁掉的顶部行数，并把越界坐标限制在可见帧内。runtime 使用内部 insertion effect，每次组件渲染后提交；props 相等跳过渲染时保留上次位置。顺序同 React `commitMutationEffectsOnFiber`：本节点的删除清理、子节点、本节点的 insertion effect，最后执行的声明生效；卸载任一调用过此 hook 的组件会清空共享位置，不检查当前所有者。不公开 insertion-effect hook。光标随帧节流；帧和光标未变时不写字节，光标变化时写移动与显隐序列。非交互、`testing::render`、`render_to_string` 忽略光标。同 Ink，首帧起隐藏光标；声明了位置时移过去并显示，下一次写帧前隐藏，输入法候选框靠它定位。
- **非 TTY / CI**：`interactive` 默认取"不在 CI 且 stdout 是 TTY"，选项可以覆盖，同 Ink。非交互时不刷新画面：提交时写新的 Static 条目，退出时写最后一帧加换行。挂载时读取尺寸，后续由 resize 事件更新，未知维度用 80×24 回退；根宽度为列数。非 TTY 被选项强制为交互模式时不裁剪高度，不视作满屏，不使用同步输出或备用屏。CI 与颜色环境检测归驱动，在 render 解析一次，核心接收解析值。CI 同 is-in-ci：`CI` 或 `CONTINUOUS_INTEGRATION` 存在且值不是 `0`、`false`。颜色未被选项覆盖时，按 chalk vendored supports-color 的 fd 1 分支顺序：FORCE_COLOR、Azure DevOps、非 TTY、TERM=dumb、Windows、CI vendors、TeamCity、COLORTERM=truecolor、具名 TERM、TERM_PROGRAM、TERM patterns、COLORTERM 存在、保底级别。数值 FORCE_COLOR 指定 0–3（大于 3 取 3），false 禁色，true / 空串提供 Basic 保底但允许探测更高级别；其他值同 chalk 按 `parseInt` 取前缀，0 禁色，正数为保底级别，无法解析或为负时仍算强制、保底为 0。颜色分支中的 CI 判断是变量存在，不使用 is-in-ci 的真假规则。0/1/2/3 映射 None/Basic/Ansi256/TrueColor。检测函数接收 env lookup 和 Windows 平台标记，不修改进程环境。Windows 分支在最低版本 1809 下返回 TrueColor，不查询系统版本；非 TTY 和 TERM=dumb 的判断在它之前。不移植 argv flags（库不读取宿主程序参数）。
- **外部输出**：`Scope::use_stdout()`、`Scope::use_stderr()` 返回 `Copy` 的 UI 线程句柄 `OutputWriter`，提供 `write(&str)`，按 runtime id 经状态句柄共用的线程局部注册表找到驱动安装的 `WriteDestination`。render 中写入在 debug 构建下 panic；空字符串忽略；卸载完成最终绘制、开始树清理时关闭写入，清理中的写入丢弃。交互模式擦掉帧、写数据、重画最后绘制的帧及其光标，同 Ink 的 `writeToStdout` / `writeToStderr`；外部写不等窗口，不消费待绘制帧，不改窗口。stdout 用一次 write 发出 `[bsu] + clear + data + redraw + [esu]`；stderr 用 stdout 发出 `[bsu] + clear`、stderr 发出 data、stdout 发出 `redraw + [esu]`，同步序列的条件同帧输出。app 将输出状态与 Runtime 分开持有，执行用户 render、effect、cleanup 时不借用输出状态；layout effect 中的 write 在调用时到达终端，重画前一次绘制的帧，首帧绘制前没有帧可重画。非交互模式把数据原样写给目标流；`render_to_string` 没有目的地，丢弃写入。不捕获 `println!`：它直接写 fd 1，安全代码拦不住；UI 运行期间改用这两个 write。
- **带外控制写入**：`OutputWriter::write_control(bytes)` 发送帧模型装不下的转义序列（kitty 图像传输、OSC 标题、OSC 52、OSC 8），Ink 没有对应 API。检查同 `write`。字节进 `AppOutput` 的队列，在包含调用前所有状态改动的那一帧写完后原样发出，每段一次 write，不包同步序列；那一步没有帧要画（非交互，或交互且未交出终端、无待绘制帧、Runtime 无待提交工作）时在步末发出。不擦帧，不重画，不补换行，不计入帧高；打印文字或移动光标而不复原的字节会让下一帧的 diff 失步，由调用方负责。交互模式交出终端期间字节留在队列，恢复后的首帧写完再发；交出期间卸载则丢弃。卸载时在最终绘制之后、树清理之前发出；panic 卸载丢弃。`external_write` 和 `clear` 不发送队列。`render_to_string` 和 `testing::render` 丢弃。
- **透传写入**：`OutputWriter::write_passthrough(bytes)` 的时机和规则同 `write_control`，字节发给多路复用器外层的终端。挂载时有 `TMUX` 的应用把每次调用包成一条 tmux 透传序列：`ESC P tmux;`、ESC 加倍的字节、`ESC \`；tmux 需要 `allow-passthrough on`。没有 `TMUX` 时等同 `write_control`。包裹发生在字节进 `AppOutput` 队列时，是否在 tmux 里由驱动从环境读出，经 `Terminal::tmux()` 给出，runtime 不保存。ink-rs 不分块：调用方每次发一条命令，kitty 图像数据由调用方按 4096 字节分块，tmux 单条输入的上限默认 1 MB。herdr 丢弃这种透传，并从 pane 的环境里删掉 `TMUX`，所以在 herdr 里不包裹。
- **外部输出的换行**：交互模式忽略转义序列与 `\r`，在数据的最后一个文本字符不是 `\n` 时补 `\r\n`，没有文本字符时不补；Ink 都不补。重画依赖 cell diff 的第 0 列起点，缺少结尾换行会覆盖数据。数据中的 `\n` 原样写出，同 Ink：Unix raw mode 同 libuv 保留 `OPOST` 与 `ONLCR`；Windows 保留 `ENABLE_PROCESSED_OUTPUT`，不设置 `DISABLE_NEWLINE_AUTO_RETURN`，由终端回到第 0 列。测试的 `Screen` 在交给 vt100 前把 `\n` 换成 `\r\n`，模拟 `ONLCR`。非交互和 debug 模式不转换数据。
- **debug 外部输出**同 Ink：stdout write 向 `testing::render` 的 `frames()` 添加一项 `data + 累计 Static + 上次 live`；stderr write 向 `stderr()` 添加 data，再向 `frames()` 添加 `累计 Static + 上次 live`。顺序按写入时刻记录，每一步结束时并入 `frames()` / `stderr()`；在步进之外经保留的句柄写入的内容，下一步后出现。`TestTerminal::output()` 按写入顺序记录 stdout / stderr，`writes()` 与 `stderr()` 分流读取。
- **恢复终端**：退出、panic、终止事件时还原输入模式、关闭 bracketed paste、弹出 kitty flags、显示光标、退出备用屏、把光标移到帧下方；panic 和终止事件经恢复快照完成，开启了粘贴或 kitty 时快照含 `CSI ?2004l`、`CSI <u`。Unix 的 signal-hook iterator 线程向所有 live app 转发 SIGINT / SIGTERM / SIGHUP 与 SIGWINCH；Windows 的 console ctrl handler 转发控制事件，尺寸变化用轮询。所有 app 位于 wait_until_exit 时，各 UI 循环卸载并运行清理，最后一个注册移除后经 sys 终止进程（Unix 用 `emulate_default_handler` 按原信号终止，Windows 用 `STATUS_CONTROL_C_EXIT` 退出），其他已卸载循环等待进程结束；保留首个终止信号。任一 app 位于循环外（包括 render 到 wait 之间、手动卸载期间或同线程嵌套 wait 暂停的外层循环），平台事件线程恢复所有 live app 后终止，不运行树清理。最后一个 app 卸载后将 default_signals 置为 true：Unix 打开 `register_conditional_default` 的条件，Windows handler 返回 FALSE，系统执行默认处理。事件设施保持安装，避免 Unix 的 signal-hook unregister 后残留 handler 吞掉信号。正常卸载先提交未完成工作、写待绘制帧，再运行树清理，最后恢复终端；无待绘制帧时不重画，因此 clear 后卸载不恢复内容。panic 中的 Drop 跳过提交和 paint，先恢复终端再运行清理，避免清理中的 panic 导致 abort 前终端未恢复。每次 Terminal 操作连同其完成后的恢复字节交给驱动，写入与发布快照共用锁：主屏来自不改变 Output 状态的 done 字节，备用屏为 `CSI ?1049l CSI ?25h`；stderr 外部写跨流阶段还结束未闭合的同步更新。首个恢复者写快照并标记已恢复，后续写入（包括 unwinding Drop）丢弃。测试 Terminal 忽略快照。render 安装一次进程 panic hook，先恢复当前 panic 线程拥有的 app，再调用原 hook；hook 先释放 stdin，再写恢复快照，快照的锁被 panic 打断的操作持有时跳过，由 unwinding 的 Drop 写出；其他线程 panic 不碰 UI 终端。`cfg(panic = "abort")` 下恢复所有 live app。会被捕获的组件 panic 不经过这条路径，见[退出](#退出)。
- **挂起**：`AppHandle::suspend_terminal(f)` 对应 Ink 的 `suspendTerminal`，只有闭包形式，没有返回句柄的形式。请求在下一步末尾生效：取消 kitty 查询，重置输入解析器，写出关闭粘贴、弹出 kitty flags 的字节；交互模式另擦除帧并显示光标，备用屏写 `CSI ?1049l CSI ?25h`。这些字节一次写出，恢复快照置空，最后退出 raw mode。同步驱动在调用 `wait_until_exit` 的线程上运行 f；tokio 驱动经 `spawn_blocking` 运行 f，同线程的任务继续运行。f 运行时该 app 不算在循环中，终止信号恢复终端后按默认效果终止。f 返回后重新查询尺寸，回到备用屏、推回 kitty flags，丢弃 Output 的状态，下一步重画；raw mode 与粘贴按持有者恢复，同 Ink。挂起不暂停提交与 effect。交互模式挂起时不绘制、丢弃外部写、`clear` 不写字节，resize 只更新尺寸，卸载不写恢复字节；挂起期间新增的 Static 条目在恢复后写出。非交互模式交出输入，输出照写；Ink 不交出输入。已有请求或挂起中再请求返回 `AlreadySuspended`；卸载开始后丢弃 f、不调用，返回 `Ok`。挂起不冲刷节流中的帧。f 中的 panic 向上展开，终端停在交出状态。
- **Ctrl+Z**：`Options::suspend_on_ctrl_z` 默认开启，在 Windows 无效；Windows 挂载时清除此选项，0x1A 交给 `use_input`，没有 job control 或停止／继续事件源。下述进程组与信号规则适用于 Unix。Ink 没有这一项，Node 的 raw mode 把 Ctrl+Z 当普通按键交给 `useInput`。开启时，按下或重复的 Ctrl+Z（不带 shift、meta、super、hyper，不在粘贴中）不交给 `use_input`，app 向本进程组发 SIGTSTP，同终端的挂起键；cooked 模式下终端驱动发出 SIGTSTP。信号线程收到 SIGTSTP 后向位于循环中的 app 发 Stop，各循环交出终端（步骤同挂起，没有回调）并应答，全部应答后用 `emulate_default_handler` 停止进程。render 与 wait 之间的 app 不在循环中，不交出终端便停止。SIGCONT 让因 Stop 挂起的 app 恢复并重画；外部的 SIGSTOP 不经过信号线程，之后的 SIGCONT 不重画。进程组是会话首进程的组时（终端或 tmux 直接启动的程序），没有 shell 能让它继续，忽略 SIGTSTP；孤儿进程组的完整判断要读其他进程的父进程，不做。`bg` 不在支持范围：后台进程恢复终端模式时收到 SIGTTOU 而停止。

## 输入

驱动读 stdin 字节，核心解析与分发，终端模式经 `Terminal` 切换。hooks 与 Ink 对应：`use_input`、`use_paste`、`use_focus` / `use_focus_manager`、`use_app`、`use_window_size`、`use_box_metrics`、`use_animation`（所有动画共用一个定时器）。

- **解析**移植 Ink 的 `input-parser.ts` 与 `parse-keypress.ts`，放在核心的 `input` 模块，没有 I/O。解码同 Node 的 `setEncoding('utf8')`：读取末尾不完整的 UTF-8 序列留到下一次，非法字节换成替换字符。字节切成文本、按键与粘贴事件。BS、DEL、Ctrl+C、CR 与 TAB 各成一个事件，其余字符连成文本；Ink 把 CR 与 TAB 留在文本里照顾粘贴，一次读到的 Enter 与 Tab 会变成文本，ink-rs 拆开，粘贴的文本靠 bracketed paste 保持完整。末尾单独的 Esc 等 20ms 再交出，截止时间并入 `render_if_due` 返回的唤醒时间。终端回复的字符串序列（APC `ESC _`、DCS `ESC P`、OSC `ESC ]`、SOS `ESC X`、PM `ESC ^`，以 ST 或 BEL 结尾）是单独的事件，不进入按键解码、Ctrl+C、焦点和回调，图形探测之外丢弃。载荷限于可打印 ASCII，遇到别的字节（终结符以外的控制字符、非 ASCII、不构成 ST 的 ESC）时放弃，按按键解析，Alt+P、Alt+_ 这类以引导符开头的按键不被吞掉。没有终结符的序列沿用 Esc 的 20ms 超时：超时后交出 ESC 加引导符，作为一个 Alt 按键，余下的字节按普通输入解析。粘贴的内容不解析。不移植 Ink 接收 `Uint8Array` 的分支：App 设置编码后不会走到它。
- **按键**：回调收到 `(&str, KeyEvent)`。`KeyEvent` 的字段是 Ink `Key` 的 snake_case，两个 Rust 关键字改名：`return` 叫 `enter`，`super` 叫 `super_key`。kitty 事件的类型为 `Some(KeyEventType::{Press, Repeat, Release})`，传统按键为 `None`。
- **分发**：`App::input(bytes, now)` 按事件顺序处理 kitty 查询回复、粘贴、Ctrl+C、Esc 与 Tab 的焦点切换，再按注册顺序调用活跃的 `use_input` 回调。一次调用产生的更新合并成一次提交。`Scope::use_input(active, handler)` 在 passive effect 中按 `active` 注册，注册顺序即 effect 顺序；回调取最近一次渲染的闭包。同一次读取的多个按键调用同一次渲染的闭包，按键各自改状态时用 `update`，`set(f(get()))` 读到的是渲染时的值。没有持有者时丢弃输入；回调请求退出后，本次读取余下的事件不再分发。Tab、Esc、Ctrl+C 在 runtime 处理，`testing::render` 与 `testing::mount` 都能用。`exit_on_ctrl_c`（默认开启）时，按下或重复的 Ctrl+C 退出应用，不交给 `use_input`；松开交给回调。Ctrl+Z 见[终端输出](#终端输出)。
- **raw mode** 由活跃的 `use_input`、`use_paste`、`use_focus` 持有，同 Ink。持有者从 0 变 1 时开启；回到 0 时清空解析状态，模式切换推迟到提交结束，同一次提交中先停用再启用的 hook 不切换模式，对应 Ink 推迟到微任务的关闭。stdin 不是 TTY 时，`use_input` 与 `use_paste` 让应用以 `Error::Io`（`ErrorKind::Unsupported`）退出，对应 Ink `setRawMode` 抛出的错误；`use_focus` 不开启 raw mode，不报错，同 Ink 的 `isRawModeSupported` 判断。图形探测进行时 raw mode 打开，与持有者无关，见下面的图形探测。
- **焦点**：`Scope::use_focus(FocusOptions) -> bool` 返回本 hook 的 ID 是否有焦点。`FocusOptions { active, auto_focus, id }` 默认活跃、不自动聚焦、生成 ID；生成的 ID 为 `ink-rs:focus:` 加 runtime 内计数，代替 Ink 的 `useId`。`Scope::use_focus_manager()` 返回 `Copy` 的 `FocusManager`，按 runtime id 找到状态，提供 `enable_focus`、`disable_focus`、`focus_next`、`focus_previous`、`focus(&str)` 与 `active_id()`；render 中修改在 debug 构建下 panic，卸载开始后修改无效、读取为 `None`。状态在 Store：按注册顺序的条目、当前 ID、是否启用，以及调用过这两个 hook 的组件的弱引用；ID 变化时标脏这些组件并唤醒驱动。注册与激活在 layout effect 中执行，自动聚焦出现在首帧；Ink 在 passive effect 中执行，测试读稳定后的帧，看不出差别。条目按 hook 实例区分：重复 ID 共用焦点，导航取每个 ID 的第一个条目及其活跃标志，卸载与停用只改本 hook 的条目；Ink 按 ID 改写所有同名条目。`active_id()` 读实时状态，effect 与回调看到之前的修改；Ink 读 render 快照。Tab、Shift+Tab、Esc 在 Ctrl+C 之后、`use_input` 回调之前处理，不消费按键；Ink 的 Tab 监听在挂载时的输入监听之后。规则同 Ink：忽略松开，Ctrl / Meta / Super / Hyper 屏蔽两键，Shift 屏蔽 Esc，停用焦点管理时两键与注册时的自动聚焦不起作用，`focus_next` 等显式调用不受影响。
- **粘贴**：有活跃的 `use_paste` 时写 `CSI ?2004h` 开启 bracketed paste，最后一个停用时写 `CSI ?2004l`，stdout 不是 TTY 时不写。`use_input` 与 `use_paste` 共用一套注册，raw mode 持有者数是两者之和。粘贴模式与 raw mode 一样在步骤结束时比较：开启在 raw mode 之后，关闭在 raw mode 之前；同一次提交中替换 `use_paste` 不写模式字节，Ink 在各 effect 中切换。粘贴内容交给 `use_paste`，不进入 `use_input`；没有 `use_paste` 时，粘贴内容作为一次输入交给 `use_input`，其中的 `\x03` 不触发退出。
- **kitty 键盘协议**：`Options::kitty_keyboard` 对应 Ink 的 `kittyKeyboard`，取值 `KittyKeyboard::{Disabled, Auto(KittyFlags), Enabled(KittyFlags)}`，默认 `Disabled`。`KittyFlags` 用 `|` 组合，默认 `DISAMBIGUATE_ESCAPE_CODES`；`REPORT_ASSOCIATED_TEXT` 的值为 24，含全键报告位，代替 Ink 的 `resolveFlags`。`Enabled` 要求 stdin 与 stdout 都是 TTY，写 `CSI >{flags}u`；`Auto` 还要求交互模式，先写 `CSI ?u` 查询，200ms 内读到回复才推入 flags。截止时间不并入唤醒时间，读到回复时与每步开始时检查是否过期。回复只在 raw mode 期间读到，在丢弃过滤之前消费，runtime 把读到回复告诉 App；粘贴中的回复按输入处理。恢复字节依次为 `CSI ?2004l`、`CSI <u`，再接屏幕恢复或最终帧。
- **图形探测**：`Scope::use_graphics()` 返回 `Graphics { support, cell_pixels }`。`support` 为 `Probing`、`Kitty`（kitty 图形协议，含 Unicode 占位符）或 `Unsupported`；`cell_pixels` 是一个单元格的像素宽高，未知为 `None`，与 `support` 无关。Ink 没有这个 hook。
  - **发出**：app 中第一次调用时探测，每个 app 一次，用不到图片的应用不付往返的代价。hook 在 render 中登记请求；app 提交、切换 raw mode 之后经 Terminal 写三条查询：kitty 查询 `ESC _ G i=31,s=1,v=1,a=q,t=d,f=24;AAAA ESC \`、`CSI 16 t`、DA1 `CSI c`，不包同步序列。挂载时有 `TMUX` 的应用把 kitty 查询和 DA1 包成透传，`CSI 16 t` 由 tmux 回答，不包裹。
  - **回复**：`i=31` 且载荷为 `OK` 的 kitty 回复记为支持；`CSI 6;h;w t` 给出像素尺寸；DA1 的回复结束探测，终端按顺序回答，DA1 排在最后。查询发出 1 秒后按收到的回复结束，截止时间并入唤醒时间。回复在丢弃过滤之前消费，不进入 `use_input`；kitty 键盘协商与探测可以重叠。
  - **输入**：探测期间 raw mode 打开，结束后按持有者恢复。没有输入持有者时，非回复的输入丢弃，Ctrl+C 和 Ctrl+Z 按选项生效，代替 raw mode 关掉的信号。交出终端（`suspend_terminal`、Ctrl+Z）结束探测，结果为 `Unsupported`。
  - **不探测**的应用首次调用得到 `Unsupported`，不写字节：非交互、stdout 或 stdin 不是 TTY、`testing::render`、`render_to_string`，以及回答 kitty 查询而占位符不可用的终端（`TERM_PROGRAM=WezTerm`，或有 `KONSOLE_VERSION`）。环境由驱动在挂载时读取，`testing::mount` 不读进程环境。
  - **像素尺寸**：驱动给出的尺寸优先，resize 时重读；驱动不知道时用 `CSI 16 t` 的回复。Windows 和 `TestTerminal` 报告未知。
  - 值变化时，调用过这个 hook 的组件标脏，同 `use_window_size`。herdr 自己回答三条查询、填写像素字段，不需要专门的代码；herdr 里跑 tmux、tmux 里跑 herdr 时识别不出。见[记录 069](decisions/069-terminal-graphics.md)。
- **读线程**属于进程实现：raw mode 开启时启动，关闭时停止。Windows 使用 console input records 和唤醒事件，见[平台](#平台)。Unix 线程持有 stdin 的复制 fd，用 `filedescriptor::poll` 同时等待 stdin 与一个 socketpair 的唤醒端，不设超时；每次可读调用一次 `read`，把字节作为消息交给驱动。停止时向唤醒端写一个字节并 join。fd 保持阻塞，不设 `O_NONBLOCK`：这个标志在打开文件描述上，与 shell 和其他读者共享。读到 EOF 后线程结束，应用继续运行；下一次开启 raw mode 时重启读线程。`EINTR` 之外的 poll / read 错误写入应用的首个 I/O 错误，循环以 `Error::Io` 结束。锁顺序为恢复状态、stdin 生命周期、输入路由、错误槽；读线程只取后两个，join 时不持有它们。驱动合并队列里的输入消息，再调 `App::input`，同 Ink 循环 `read()` 直到为空。
- **termios** 在 Unix 用 rustix 设置，标志同 libuv 的 raw mode：清除 `BRKINT ICRNL INPCK ISTRIP IXON` 与 `ECHO ICANON IEXTEN ISIG`，设置 `CS8` 与 `ONLCR`，保留 `OPOST`，`VMIN=1`、`VTIME=0`。开启前保存原设置，关闭和恢复终端时还原。同 libuv，开启与正常关闭用 `TCSADRAIN`，panic 与信号的恢复用 `TCSANOW`。最后一个持有者释放时先停读线程再还原，否则已就绪的 read 会等一整行。恢复终端时关闭该应用的开启入口，信号线程与 UI 线程竞争时不会重新开启。
- **多个应用**：进程内的 raw mode 持有者列表与读线程各一个，输入交给最近开启 raw mode 的应用，它释放后交给前一个。

## 退出

同 Ink 的 `useApp().exit`。`AppHandle` 提供三个方法：`exit()` 的结果为 `Ok(None)`；`exit_with(value)` 为 `Ok(Some(value))`，值类型为 `Box<dyn Any + Send>`，调用方 downcast，Ink 的结果可以是任意值；`exit_with_error(error)` 为 `Err(Error::App(..))`。`Error` 有三个变体：`Io(io::Error)`、`App(Box<dyn std::error::Error + Send + Sync>)` 与 `Panic(PanicError)`。

- 第一次退出请求生效；开始卸载后的请求，包括清理函数中的调用，被忽略。
- 退出不在调用内完成。下一步依次：关闭 raw mode，写待绘制帧，运行树清理，恢复终端（关闭粘贴、弹出 kitty flags、显示光标、退出备用屏），最后把结果交给 `wait_until_exit`。
- `wait_until_exit` 阻塞时，外部经保留的 `AppHandle` 调 `exit()` 结束应用，对应 Ink 测试中的外部 `unmount()`。Ctrl+C 与自然结束的结果为 `Ok(None)`；I/O 错误为 `Err(Error::Io(..))`。
- 结果在 App 中合并，取第一个失败：退出错误，其次是卸载中的 I/O 错误，最后是退出值。组件 panic 替换待处理的 `Ok` 退出，不替换先前的 `Err`。
- `TestApp::exit_result` 读取合并后的结果，`take_exit_result` 取走结果，连同退出值与 panic payload 的所有权。`testing::render` 没有 App，退出请求不生效。
- 退出错误不改变进程退出码，由应用决定如何报告，同 Ink。
- **组件 panic**：同 Ink 的 ErrorBoundary，live app 捕获 reconcile、insertion effect、layout effect（含清理）、passive effect 与 Static 退场后清理中的 panic，`catch_unwind` 包住这些区域。render 或 layout effect 中的 panic 让本轮提交换上空根，比对时退场出错的子树，清理在捕获区内运行；passive 中的 panic 留待下次提交换树。记录第一个 panic，丢弃之后的。App 用错误界面重渲染根，以 `Err(Error::Panic(..))` 请求退出，下一步走正常卸载：关闭 raw mode、写出错误界面、清理、恢复终端。首次渲染的 panic 在 render 的第一步卸载，`wait_until_exit` 返回存下的结果。错误界面提交中的 panic 继续 unwind，不循环。
- **`PanicError`** 保存消息（`&str` / `String` payload，其他类型为 `Box<dyn Any>`，同 std 的 hook）、位置与原 payload；`resume_unwind(e.into_payload())` 得到 panic 的退出码 101。`State`、`OutputWriter`、`FocusManager` 中会 panic 的方法标 `#[track_caller]`，位置指向调用者。
- **错误界面**同 Ink 的 ErrorOverview：红底 ` ERROR ` 标签与消息、暗色位置、出错行上下各三行的源码摘录（读取失败时省略）。设置 `RUST_BACKTRACE` 或 `RUST_LIB_BACKTRACE` 时附调用栈：去掉 std、core、alloc 与两个 hook 的帧，在 runtime 调用组件处截断，每帧一个 `Text`，长符号在一段内折行。它是卸载时的最终帧，备用屏退出时随屏幕丢弃，同 Ink。
- **不捕获**：输入与粘贴回调、`AppHandle::update` 闭包、Transform，同 Ink；卸载中的清理，Ink 丢弃这里的错误，ink-rs 让 panic 传出；`testing::render` 与 `render_to_string`，panic 让 Rust 测试失败；`panic = "abort"`。这些 panic 走恢复终端的 hook 路径。
- **两个 hook 协作**：App 创建时安装一次核心 hook（`runtime::panic`）。panic 发生在捕获区内时，核心 hook 记录位置与 backtrace 后返回，不调用原 hook，std 的消息不覆盖帧；驱动 hook 见到同一条件，不恢复终端，交给前一个 hook。两者共用安装锁，安装顺序任意。unwinding 中再次进入 hook 是 Drop 中的 panic，之后 std 会 abort，这次走未捕获路径，驱动先恢复终端。组件自己的 `catch_unwind` 接住的 panic 不打印 std 的消息，但置位 unwinding 标记，同一捕获区内之后逃出的 panic 走未捕获路径。`use_future` 的任务每次 poll 也在核心捕获区内，见 tokio 驱动。

## 测试

Ink 的测试是规格，分四层：

1. `render_to_string`：纯函数，M0 的验收。
2. `testing::render`：对应 ink-testing-library 的 debug 模式，每渲染一帧记录一帧，不节流（M1）。模拟按键在 M3，假时钟随节流在 M2。
3. 输出字节交给 vt100 模拟器执行，检查屏幕和 scrollback（M2）。Ink 锁定字节的测试，改为检查 Ink 记录下来的字节在 vt100 上产生的屏幕。vt100 按 char 计算宽度，resize 时不重排、不把行推进 scrollback，待换行状态下的 EL 不擦最后一格。`tests/common::Screen` 在主屏宽度收缩时补上重排模型：按 scrollback、屏幕的顺序读取行，前一行的 wrap 标记决定是否续接；光标和最后非空行下方的空行不算逻辑行。以新列数重建 parser，由 vt100 换行并标记 wrap，保留文本、宽字符与字形样式；光标按逻辑行内的格偏移映射，超出内容时取最后一个物理行的末尾，同 tmux，保留显隐。旧屏首行作为锚点，重排增加的行占用下方空行（tmux 以屏幕底部为锚，模型不跟随），光标超出屏底时按使其可见所需的行数滚动。备用屏不重排，宽度增长不拼回折行。宽高收缩按重排、行收缩的顺序计算；行收缩使用 Ink `reconstruct-terminal` 的 xterm 模型，丢弃光标下方行后把顶部行推进 scrollback；行增长从 scrollback 拉回至多增长行数的行，光标随之下移，备用屏不拉回。vt100 对宽字符跨过行尾剩余一格的换行不设 wrap 标记，一列屏幕写双宽字符会触发下溢；这两种情况不在模型支持范围，交给真实终端验收。
4. portable-pty 跑示例程序，测 raw mode、Ctrl+C、退出码、挂起（M3 起）。`tests/pty.rs` 不用 libtest：同一个可执行文件按环境变量作为 fixture 运行，主进程在 100×24 的 PTY 中启动它并检查输出标记与退出码。

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

### 示例

- `examples/` 移植 Ink 的示例，目录名同 Ink（kebab-case），`cargo run --example use-input` 对应 Ink 的 `npm run example examples/use-input`。文件开头注明移植的 Ink 示例。
- `frame-rate` 是 ink-rs 自有的示例，不是移植，显示 `max_fps` 下实测的帧率和帧间隔（见[记录 065](decisions/065-frame-rate-example.md)）。文件开头注明。
- 有测试的示例用 Cargo 的多文件布局：`main.rs` 放 `main`，`app.rs` 放组件、reducer 和文本逻辑，只用公开 API。`tests/examples.rs` 以 `#[path]` 引入 `app.rs`，在进程内挂载，不开 PTY；需要真实终端的示例（resize、终端交接、进程自然结束）在 `tests/examples_pty.rs` 中运行。没有测试的示例是单个文件，同 `counter.rs`。
- 示例用 `rsx!` 建树，`Cargo.toml` 为每个示例和两个示例测试声明 `required-features = ["macros"]`。
- 依赖 tokio 的示例（jest、subprocess-output）另声明 `tokio` feature，`main` 为 `#[tokio::main(flavor = "local")]`，异步工作放在 `use_future` 中。外部数据经 channel 交给组件：subprocess-output 的 `app` 接收 `UnboundedReceiver<Chunk>`，`main` 从子进程读取，测试直接送入数据块。
- 周期刷新用 `use_animation`，测试用 `advance` 推进。按 tick 计数的状态由 `frame` 推出；其他状态在以 `frame` 为键的 `use_effect` 中每交付一帧走一步，迟到的唤醒跳过的格点不补走。示例在注释中写明用哪一条。`counter.rs` 保留线程加 `AppHandle` 的写法。
- `useReducer` 改为 `State<T>` 加 `fn reduce`，`Math.random` 改为本地的 xorshift，faker 改为固定数据。
- aria、select-input 移植画面，屏幕阅读器部分随该模式延后。`concurrent-suspense`、`suspense`、`use-transition` 依赖 React 并发，`router` 依赖 React Router，不移植。incremental-rendering 去掉 `incrementalRendering` 选项：ink-rs 在各模式下都按格子差分。

## 里程碑

| 阶段 | 内容 | 验收 |
|---|---|---|
| M0 | View/Text、布局、文本、边框、`render_to_string`；元素是数据，没有运行时 | 移植的 Ink 布局和文本测试通过 |
| M1 | 组件树、hooks、按 key 比对、调度、元素尺寸测量、测试驱动与 `testing::render` | 断言重渲染次数，空闲时零渲染 |
| M2 | 两种输出模式、Static 跨帧、`max_fps` 节流、光标、非 TTY、外部输出、resize 与 `use_window_size`、终端恢复、同步驱动 | vt100 场景测试 |
| M3 | 输入、焦点、粘贴、kitty 键盘协议、退出 | PTY 测试 |
| M4 | 动画、`use_box_metrics`、panic 时的错误界面、挂起恢复、tokio 驱动；移植 Ink 的示例 | 逐个对照示例 |
| M5 | 宏 DSL | 宏与 builder 的渲染结果相同 |

屏幕阅读器模式延后，不排期，涉及的 Ink 测试记在 [ink-tests.md](ink-tests.md)。

## 平台

支持 Linux、macOS 和 Windows 10 1809（build 17763）及之后版本，Windows 覆盖 conhost 与 Windows Terminal。`render`、`Instance` 与 tokio 驱动共用调度、输入路由、恢复注册表和清理流程；系统接口放在 `driver/sys`，Unix 用 `sys/unix.rs`，Windows 用 `sys/windows.rs`。

- **unsafe 边界**：Windows console FFI 是项目的 unsafe 豁免，unsafe 代码限定在 `src/driver/sys/windows.rs`。crate 在非 Windows 使用 `forbid(unsafe_code)`，在 Windows 使用 `deny(unsafe_code)`；`sys/mod.rs` 的 `mod windows` 使用 `allow(unsafe_code)`。每个 unsafe 块注明句柄、缓冲长度或输出参数的安全条件。系统调用使用 windows-sys 0.61，按使用的接口开启 feature。
- **输出**：复制 stdout / stderr 句柄，绕过 std 的 LineWriter，让同步帧作为一个缓冲进入 write。`GetConsoleMode` 成功时，整次 write 的字节转 UTF-16，用 `WriteConsoleW` 循环写完；非法 UTF-8 替换为 U+FFFD。管道和文件用 File 写原字节。不调用进程范围的 `SetConsoleOutputCP`。每个进程一次为 stdout / stderr 添加 `ENABLE_VIRTUAL_TERMINAL_PROCESSING` 和 `ENABLE_PROCESSED_OUTPUT`，不设置 `DISABLE_NEWLINE_AUTO_RETURN`，LF 保持回到第 0 列。卸载不还原输出模式，同 libuv 的 `uv__determine_vterm_state`，由 shell 重置。
- **TTY 与颜色**：stdout 要求 `GetConsoleMode` 成功且 VT processing 开启；legacy console 不算 TTY，默认走非交互、无颜色路径。stdin 要求 `GetConsoleMode` 成功。msys / mintty 管道不算 TTY，不使用 std 的 `IsTerminal`，同 libuv。颜色分支接受 Windows 平台标记，1809 超过 supports-color 的 build 14931 门槛，返回 level 3；不读取 Windows 版本，FORCE_COLOR、非 TTY 和 TERM=dumb 保留原优先级。
- **输入**：SavedMode 是 console input mode 的 u32。开启 raw mode 保存原值，设置 `ENABLE_VIRTUAL_TERMINAL_INPUT`，不加其他标志；无 line / echo / processed input，Ctrl+C 作为 0x03 进入输入。同 libuv `UV_TTY_MODE_RAW_VT`，但不加 libuv 用于尺寸记录的 `ENABLE_WINDOW_INPUT`，尺寸由轮询得到；不设置 `ENABLE_EXTENDED_FLAGS`，quick-edit 保留用户选择。关闭与 emergency 恢复使用保存值，不区分两条路径。
- **输入线程**：复制 stdin 句柄，创建 manual-reset event，启动 ink-stdin。`WaitForMultipleObjects` 等待输入与停止事件；事件触发时退出。输入可读时用有界缓冲调用 `ReadConsoleInputW`，保留 KEY_EVENT 中字符非零的 key-down，以及 VK_MENU 的字符非零 key-up（Alt+numpad 组合结果），忽略其他记录。解码函数保存跨读取的高代理项，将 UTF-16 转 UTF-8，按记录的重复次数发送字符；VT 输入的转义序列按字符到达，不为 Alt 添加 ESC。等待或读取失败经 `stdin::ended` 传递错误，console 没有 EOF。停止时 SetEvent 并 join，线程 panic 转 I/O 错误。
- **尺寸**：`GetConsoleScreenBufferInfo` 读取 stdout，列数用 `dwSize.X`，行数用 `srWindow.Bottom - srWindow.Top + 1`，同 libuv `uv__tty_console_signal_resize`；失败返回 None。进程内 ink-resize 线程每 100ms 查询尺寸，变化时调用共享 resize 通知。不用 `SetWinEventHook`：该方案需要 user32、消息循环和 ntdll 未公开的 `NtQueryInformationProcess` 查找 conhost。
- **控制事件**：进程内一次注册 `SetConsoleCtrlHandler`，static 保存 default_signals。没有 live app 时返回 FALSE，让系统执行默认处理；有 live app 时将 ctrl type 交给 `process::signal`，然后阻塞 handler 线程，不返回。CLOSE / LOGOFF / SHUTDOWN 在 handler 返回后结束进程，返回会跳过 UI 清理；系统的关闭超时可能中断清理。最后一个 UI 注册移除时退出；没有可运行清理的循环时，`process::signal` 恢复终端后退出。Termination 是 u32 ctrl type，`terminate` 用 `std::process::exit` 返回 `STATUS_CONTROL_C_EXIT`（0xC000013A）。
- **job control**：Unix 的 JOB_CONTROL 为 true，Windows 为 false。挂载时按该能力清除 `suspend_on_ctrl_z`；Windows 的 stop_process 无操作，Ctrl+Z 交给 `use_input`，同 Node。闭包形式的 `suspend_terminal` 保留，不依赖 job control。
