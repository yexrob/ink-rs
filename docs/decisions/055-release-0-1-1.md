# 发布 0.1.1

- 日期：2026-10-21
- 状态：已实施
- 提交：`7eb03a5` build: keep CI files out of the package；`a6cea3a` chore: release 0.1.1
- Brief：无，主 agent 自己做

## 背景

Windows 支持在 `windows-support` 分支上通过三平台 CI（见 [054](054-conpty-tests-ci.md)）。Claude 问用户：要不要把 main 快进到 `windows-support` 并推送，之后要不要发 crates.io 新版本，并说明它不会自己发布。用户回复：「发布」。

## 决策

- 合进 main 并发布。用户决定（「发布」）。
- 只升补丁号到 0.1.1：公开 API 没有变化，Windows 上的 `render`/`Instance`/`tokio` 是新增。Claude 在用户的发布指令下决定。
- `ink-rs-macros` 自 v0.1.0 起没改过，保持 0.1.0，不重新发布。
- `.github/` 和 `.gitattributes` 不打进 crate，和原有排除 `.gitignore`、`CLAUDE.md` 是同一规则。
- 顺序：main 的 CI 全部通过后再打标签和发布。
- 同 v0.1.0，不建 GitHub Release。

## 方案

主 agent 自己执行：

1. 查 `55f4f55` 那一轮 CI 的失败原因，处理后重跑。
2. 在 `Cargo.toml` 的 `exclude` 加上 `/.gitattributes`、`/.github`，用 `cargo package --list` 核对。
3. 升版本号，单独提交。
4. 本地跑 fmt、clippy、全量测试。
5. main 从 `379f75b` 快进到 `a6cea3a` 并推送，等 main 的 CI。
6. 打 `v0.1.1` 标签并推送，执行 `cargo publish -p ink-rs`。

## 结果

- `55f4f55` 那轮 CI 失败的原因是 GitHub 的 macOS runner 排不上队（「not acquired by Runner」），不是代码问题。重跑失败的 job 后 7 个 job 全部通过。
- 第一次提交 exclude 时用了 `git commit -o Cargo.toml`，把已改的版本号一起带进去了。推送前发现，软回退后拆成两个提交：`7eb03a5` 只改 exclude，`a6cea3a` 改版本号和 `Cargo.lock`。
- 等待 CI 的脚本用短 SHA 查 run，没取到 id，失败的是脚本，CI 本身正常；改为盯 run 39520876034。
- main 的 CI 在 `a6cea3a` 上 7 个 job 全部通过。`v0.1.1` 标签推送，ink-rs 0.1.1 发布到 crates.io，是第一个支持 Windows 的版本。docs.rs 的 targets 含 `x86_64-pc-windows-msvc`（由 [053](053-windows-driver.md) 加入）。
- 远端的 `windows-support` 分支合进 main 后要不要删，交给用户定，记录中未见答复。
