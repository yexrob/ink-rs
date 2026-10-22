# `Static::from_fn`：按下标渲染应用自己存的条目

- 日期：2026-10-22
- 状态：已实施
- 提交：`6159c4a` feat: add Static::from_fn for items the app stores
- Brief：无，主 agent 自己做

## 背景

eink-rs 的 Markdown 流式组件（eink-rs 的记录 018）把定稿的段交给应用的 `Static`。一个 1 MB 的回答约 2 万段，代码块按行定稿时，流式期间每帧有新段。派去写它的 Codex 停下报告：`Static::new` 收 `Rc<[T]>`，应用的历史每增长一次，就要复制一份新的切片，开销随历史长度增长。

Claude 测了每帧克隆历史、转成 `Rc<[T]>` 的开销：2 千项 5µs，2 万项 61µs，10 万项 377µs。Ink 的应用每次追加复制数组，即 `[...items, item]`。Claude 建议接受复制、记为已知限制，另一个选项是 ink-rs 加 `Static::from_fn(len, render)`，要发布 0.1.5。用户选「ink-rs 加 Static::from_fn」。

## 决策

- 用户定（「ink-rs 加 Static::from_fn」）：加 `Static::from_fn(len, render)`，发布 0.1.5，发布另开记录。
- 以下由 Claude 按同一授权定：
  - 签名 `pub fn from_fn(len: usize, render: impl Fn(usize) -> Element + 'static) -> Static`。`Static` 存的是长度和按下标渲染的闭包，`from_fn` 收这两样；`new` 改为调用 `from_fn`。
  - 不加测试：`new` 经 `from_fn` 构造，已有的 Static 测试覆盖这条路径。
  - 文档注释写明用法：条目放在应用自己的存储里（如 `Rc<RefCell<Vec<T>>>`），闭包按下标取；只渲染已写出之后的条目。`docs/design.md` 的 Static 一条补一句，注明 Ink 没有这个入口和原因。

## 备选与取舍

- **接受复制**：Claude 的建议，用户没有选。
- **`Static::new` 改收 `Rc<RefCell<Vec<T>>>` 之类的共享容器**：改动现有签名，绑定一种容器；`from_fn` 不限定存储。

## 方案

主 agent 自己做：`src/components.rs` 加 `from_fn`、`new` 改为调用它；design.md；fmt、两套 feature 的 clippy、全量测试；本地提交，推送随 0.1.5 交用户验收。

## 结果

- `6159c4a`：`Static::from_fn` 和文档注释，`Static::new` 经它构造；design.md 的 Static 一条补了说明。
- fmt、两套 feature 的 clippy 通过；`--all-features` 1331 个、`--no-default-features` 1297 个测试通过，和 0.1.4 相同。
- 没有推送。发布 0.1.5 另开记录，推送前交用户验收。
