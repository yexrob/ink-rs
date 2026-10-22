# 发布 0.1.5

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：无，主 agent 自己做

## 背景

eink-rs 正式实施 Markdown 流式渲染（eink-rs 的记录 018），Codex 停在 ink-rs 缺的原语上：`Static::new` 每次历史增长要复制一份 `Rc<[T]>`。Claude 问怎么处理，用户选「ink-rs 加 Static::from_fn」，这个选项写明「要改 ink-rs 并发布 0.1.5，eink-rs 再升级，然后 Codex 才能继续」。[066](066-static-from-fn.md) 加了 `Static::from_fn`。

## 决策

- 用户定（「ink-rs 加 Static::from_fn」）：发布 0.1.5。
- 以下由 Claude 按同一授权和 [064](064-release-0-1-4.md) 的先例定：
  - 发 `ink-rs` 0.1.5。`ink-rs-macros` 没有改动，维持 0.1.1。
  - 升补丁版本：新增一个构造函数，已有的 API 不变。0.x 下 Cargo 把 0.1.5 看作和 0.1.4 兼容，下游 `cargo update` 后拿到。
  - 顺序同 064：本地全量、`cargo publish --dry-run`、用户验收、推送 main、等 CI、打 `v0.1.5` 标签、`cargo publish -p ink-rs`。验收和 eink-rs 的 Markdown 正式实施一起交给用户：0.1.5 为它而发。

## 备选与取舍

- **升到 0.2.0**：0.x 下新增 API 不要求升次版本；升了的话，下游要改依赖要求才拿得到。
- **eink-rs 依赖本地路径或 git，不发版**：CI 拿不到本地路径；发布到 crates.io 的包不能依赖 git。

## 方案

主 agent 自己执行：

1. 升版本号和 `Cargo.lock`，单独提交。
2. fmt、两套 feature 的 clippy、全量测试、`cargo publish --dry-run -p ink-rs`。
3. 交给用户验收，连同 eink-rs 的 Markdown 正式实施和压测结果。用户通过前不推送。
4. 推送 main，等 CI。
5. 打 `v0.1.5` 标签并推送，`cargo publish -p ink-rs`。
