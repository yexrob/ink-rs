# 组件 panic 的捕获与错误界面

- 日期：2026-10-20
- 状态：已实施
- 提交：`1e0a4a0` feat(errors): display component panic overviews
- Brief：[m4-p2](briefs/m4-p2.md)、[m4-p2-fix1](briefs/m4-p2-fix1.md)

## 背景

M4 计划的 P2（见 031）。Ink 用内部的 ErrorBoundary 接住渲染与 effect 中的错误，画出 ErrorOverview 后以错误退出。调研发现 ink-rs 的阻塞点：驱动的 panic hook 在展开前恢复终端，之后的写入被丢弃，错误界面画不出来；std 的 hook 还会把消息打印在帧上。

## 决策

- 用户批准 M4 计划时（「可以」）同意：
  - 调用栈只在设置 `RUST_BACKTRACE` 时显示；
  - 卸载中、Transform 里、`testing::render` 里的 panic 不捕获；
  - panic 覆盖同一步里之前的 `Ok` 退出，不覆盖之前的 `Err`。
- Claude 在 brief 中定下（按 `research-errors.md` 的推荐）：
  - 新增 `Error::Panic(PanicError)`，保存消息、位置、私有的 backtrace 与原 payload；`into_payload()` 交给 `resume_unwind` 可得到退出码 101。不加新选项，不加用户错误边界，Ink 都没有。
  - 新核心模块 `src/runtime/panic.rs`：线程局部的捕获深度与「正在展开」标记。会被捕获的 panic 只记录位置和 backtrace，不调用之前的 hook；驱动 hook 对这类 panic 跳过恢复。展开中再次 panic 走未捕获路径，恢复终端。
  - `catch_unwind(AssertUnwindSafe)` 包住 reconcile、layout effect 组、passive effect 组与 `after_write` 的清理；输入、粘贴回调、`AppHandle::update`、Transform 不包，同 Ink。
  - render 或 layout effect 中的 panic 让本轮提交换上空根，旧树按正常路径退场；错误界面由 app 层的 `src/error_overview.rs` 构建，作为最终帧画出。
  - 退出优先级做成 `UpdateQueue` 上的请求，不在 App 里特判。
  - 给 `State`、`OutputWriter`、`FocusManager` 中会 panic 的方法加 `#[track_caller]`，位置指向用户代码。
  - `panic = "abort"` 下不捕获，写进文档。

## 方案

- 派给一个 Codex，与示例第一批（见 034）并行。
- 测试：`tests/errors.rs`（`errors.tsx` 6 个）、错误界面的 6 个（`error-overview.tsx`）、`render.rs` 1 个、`render_to_string.rs` 2 个、PTY fixture `exit-with-thrown-error`。不适用：unhandledRejection、另一个 realm 的错误、React 重复 key 警告。

## 结果

- Claude review 认可 hook 协作、捕获范围、换树与退出优先级。另在 tmux 中用一个临时 crate 手动检查，发现缺陷：用户的 shell 设了 `RUST_BACKTRACE=1`，错误界面列出约 60 帧调用栈，长符号分成左右两栏折行，帧高超过屏高，顶部被裁掉，ERROR、消息、位置与源码摘录都看不到。
- 退回 Codex 修（fix1）：
  - `frames()` 去掉栈中任意位置的 std、core、alloc、panic_unwind、`__rustc` 与两个 hook 的帧，同 Ink 用 stack-utils 去掉 Node 内部帧；在 runtime 调用组件处截断；保留开头的 ink-rs 帧（指出出错的 API）。
  - 每帧改为一个 `Text`，长符号在一段内折行，记为与 Ink 三个 `Text` 的差异。
  - 去掉 `#[path]` 的模块写法：`error_overview` 在 `lib.rs` 中声明，移植的 6 个测试移到 `src/error_overview/tests.rs`，删除 `tests/error_overview.rs`（Ink 的 ErrorOverview 是内部组件）。
  - `Runtime::guard` 的参数 `overview` 改名为 `rethrow` 并加注释。
- 修后在 tmux 中复查：栈只剩两帧用户代码，界面完整。全量 1270 个测试通过；`RUST_BACKTRACE` 开和关两种情况下 errors 与 PTY 测试都通过；clippy 无警告，示例能编译，`--no-default-features` 检查通过。
- 文档：[design.md 的退出一节](../design.md#退出)写入捕获范围、退出优先级、`PanicError`、错误界面、两个 hook 的协作与已知限制（组件自己的 `catch_unwind` 会让同一区域之后的 panic 走未捕获路径）；`docs/ink-tests.md` 更新计数，加「错误界面的观测边界」。
- 这次手动检查引出「卸载时最终帧超过屏高被裁掉」的问题，见 037。
- 遗留：用户在未受保护的输入回调里自己 `catch_unwind` 会触发驱动恢复（原有问题）；栈位置的 `./` 前缀。
