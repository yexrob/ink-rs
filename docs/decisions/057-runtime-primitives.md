# 运行时原语：context、`Element: PartialEq`、`Callback` 与 `use_callback`

- 日期：2026-10-21
- 状态：实施中
- 提交：无
- Brief：[context](briefs/context.md)、[props](briefs/props.md)

## 背景

讨论上层框架时（[056](056-upper-framework.md)），Claude 核对 ink-rs 的 API，找到三处缺口，store、路由和主题都卡在这里：

1. 没有 context。核心里只有一个传背景色的内建 context，用户用不到。store、路由、主题只能逐层传 props 或放全局变量。
2. children 不能当 props。props 要求 `PartialEq`，`Element` 没实现，写不出 `Layout { children }`。
3. 回调不能当 props。闭包没有 `PartialEq`，现在要包一层 `Rc` 再手写指针比较（`examples/subprocess-output/app.rs:18`、`tests/suspend_terminal.rs:18`）。

用户：「对 ink-rs可以先开始搞原语部分」。

## 决策

- **context 用 hook 形式**：`cx.provide(value)` 与 `cx.use_context::<T>() -> Option<T>`，不用 Provider 元素，不新增节点种类，rsx 不用改。上层的 `ThemeProvider` 写成收 children 的组件，在里面 `provide` 后返回 children。
- **`use_callback` 的语义**：返回跨渲染不变的句柄，调用时执行最近一次渲染传入的闭包，同 Dioxus，没有 React `useCallback` 的 deps。
- 以上两点 Claude 请用户确认，用户回复：「你来确认就行」。Claude 按自己的提议定下。
- 以下由 Claude 在方案中提出，用户没有另提意见：
  - context 按类型区分，同一类型用作两个 context 时包 newtype；没有提供者返回 `None`，不设默认值；`T: Clone + PartialEq + 'static`；最近的祖先胜出；组件读不到自己 provide 的值，同 React。
  - `provide` 的新值与上次不等时标脏所有读者。标脏走现有 `Dirty` 冒泡，memo 跳过的子树里的读者在同一次提交里重渲染，不多写一帧。
  - `Element` 按结构比较到组件边界为止：组件节点比较类型和 props（复用 `same_props`），不展开输出；`Transform`、`Static` 的函数按指针比较。children 不变时组件被跳过，比 React 多一次跳过机会，渲染结果不变。
  - `Callback<A = (), R = ()>` 包装 `Rc<dyn Fn(A) -> R>`，按指针相等；每次渲染新建的 `Callback` 互不相等，同 React 不加 `useCallback`。
  - 三项都是新增 API，可以放在 0.1.x 里发布。

## 备选与取舍

- **Provider 元素**：要新增节点种类并改 rsx。hook 形式不需要。
- **带 deps 的 `use_callback`**：句柄不变、调用最新闭包的语义下，子组件不会因父组件重渲染而重渲染，也不会调到旧闭包。
- **不加的原语**：`useSyncExternalStore` 类原语（外部 store 用 `use_state` 加 `use_layout_effect` 订阅即可，selector 放上层）；`use_reducer`、`use_ref`（`State::update` 和 `use_state(|| Rc<RefCell<_>>)` 能覆盖）；rsx 的组件语法（等上层用起来，见 [056](056-upper-framework.md)）。
- **在 examples 和 tests 里改用 `Callback`**：不改。那几个 newtype 包的是 channel 和 `FnOnce + Send`，`Callback` 不覆盖。

## 方案

两个 Codex worker 在各自的 worktree 里并行，两边在 `hooks.rs` 里各加一个 hook 种类，合并冲突小。

**context**（brief `context`）：
- 新增 `src/runtime/context.rs`；`provide`、`use_context` 各占一个 hook 槽，有带 `TypeId` 的 `Kind` 变体，纳入 debug 的 hook 顺序检查。
- 每个实例持有父实例给的祖先提供者链；实例祖先不变、hook 顺序固定，所以这条链在实例生命周期内不变。读者首次渲染时解析提供者，以 `Weak<Dirty>` 登记；提供者 cell 弱持有读者，避免 `Rc` 环。
- 标记读者不算写状态，不能触发 `state.rs:151` 的 debug panic。背景色继承保持原样，不并入新机制。
- 三个测试：memo 跳过的子树里的读者在同一次提交内重渲染，等值不触发；嵌套时取最近的，组件读到祖先的值；没有提供者返回 `None`。

**props**（brief `props`）：
- `Element`、`View`、`Text`、`Transform`、`Static` 及内部的 `Node`、`Component` 实现 `PartialEq`，规则写在 `Element` 的 rustdoc。
- 新模块导出 `Callback`：`Clone`、指针相等、不透明的 `Debug`、`call`；构造入口由 worker 选。
- `use_callback` 调用闭包前先把内部 `Rc` 克隆出来，不持有 `RefCell` 借用；卸载后句柄仍调最后的闭包，写状态被忽略。
- 两个测试：children 不变时跳过、变了重渲染；拿 `use_callback` 句柄的子组件被跳过，调用时执行最新闭包。

两边的验收都是 lib 测试、相关的集成测试目标、`--no-default-features` 子集、fmt 与两套 feature 的 clippy（props 另跑 doc 测试），不跑全量 `cargo test`。`docs/design.md` 编程模型一节由主 agent 在集成时写。

## 结果

- 两个 worker 按 brief 完成，报告无偏离。
- context worker 的做法：`Contexts` 是 `Rc` 链表，子实例创建时接收父实例的 `child_contexts`；组件首次渲染后按 hook 顺序把 `provide` 的 cell 压到链头，同类型后调用的胜出。读者挂载时解析提供者并登记 `Weak<Dirty>`。另在 `reconcile` 末尾按子节点重算 `descendants`：提供者在本次遍历中标脏读者，标记冒泡到已清零的祖先，不重算的话下一步会多走一次空提交。review 保留。
- props worker 选 `Callback::new` 作构造入口，与 builder 命名一致，不加 `From<F>`。`use_callback` 复用 `Value::Memo` 存储，hook 顺序检查靠 `Kind::Callback`。集成时判断不需要另加 `Value` 变体，保留。
- review 发现：提供者的读者列表只在值变化时剪除卸载的读者。值不变的提供者（如主题）下反复挂载、卸载读者，列表一直增长。集成时改为列表扩容前先剪除，长度受挂载中的读者数约束，摊销 O(1)。
- 两份 patch 只在 `Kind` 枚举处冲突，手工合并。
- 全量：`cargo test --workspace` 1328 通过，`--no-default-features` 1296 通过，0 失败；fmt、两套 feature 的 clippy 通过。
- `docs/design.md` 编程模型加两条：子元素和回调作 props、context。
- 两个 worktree 删除。
