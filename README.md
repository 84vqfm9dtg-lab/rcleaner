# rCleaner

`rCleaner` 是一个中文友好的本地清理与磁盘占用扫描工具，面向 macOS 上常见缓存、开发产物和应用残留。它默认偏保守：先扫描、先 dry-run、只对明确可重建的内容执行清理。

技术栈：`Tauri 2 + Rust + React + Vite + TypeScript + Material UI`

## 功能

- 扫描常见缓存和开发资源占用
- 展示可释放空间、目标路径、风险级别和分类
- 清理单个目标或一键清理可安全清理的目标
- 支持 `--dry-run` 预演，不真正删除文件
- 识别 `~/Library/Caches`、应用支持目录、容器目录中的大体积项目
- 对应用支持目录和容器根目录默认只读，避免误删业务数据
- 支持 Docker、音乐、办公软件、编辑器等常见 app 的更细目标拆分
- 支持自定义规则文件 `rules/cleaner-rules.toml`
- 原生 CLI 支持 `info`、`capabilities`、`scan`、`clean`

## 安全边界

`rCleaner` 不是任意路径删除器，也不是卸载器。

- 普通清理只处理相对安全、可重建的缓存或开发产物。
- `概览` 类目标只用于展示空间占用，不会直接删除。
- 疑似卸载残留和只读候选需要显式使用专家参数。
- 执行真实清理前建议先运行 `clean --dry-run --json`。

## 快速开始

```bash
npm install
npm run dev
```

打包桌面应用：

```bash
npm run build
```

检查 Rust 和测试：

```bash
npm run rust-check
npm run rust-test
```

## CLI 用法

开发期可以直接使用：

```bash
cargo run --manifest-path ./src-tauri/Cargo.toml -- info --json
cargo run --manifest-path ./src-tauri/Cargo.toml -- capabilities --json
```

构建后可执行文件位于 `./target/debug/rcleaner`：

```bash
cargo build --manifest-path ./src-tauri/Cargo.toml
./target/debug/rcleaner scan --json
```

### 扫描

扫描当前用户目录下的预置目标：

```bash
cargo run --manifest-path ./src-tauri/Cargo.toml -- scan --json
```

扫描指定模拟根目录，适合测试和 AI 沙盒：

```bash
cargo run --manifest-path ./src-tauri/Cargo.toml -- \
  scan --root /tmp/mock-home --json
```

### 清理

先预演全部可清理目标：

```bash
cargo run --manifest-path ./src-tauri/Cargo.toml -- \
  clean --all --dry-run --json
```

清理单个目标：

```bash
cargo run --manifest-path ./src-tauri/Cargo.toml -- \
  clean --target npm-download-cache --json
```

专家场景下允许处理只读候选：

```bash
cargo run --manifest-path ./src-tauri/Cargo.toml -- \
  clean --target app-support:com.example.oldapp --allow-readonly --json
```

### JSON 输出约定

成功时返回：

```json
{
  "ok": true,
  "command": "scan",
  "data": {
    "targetCount": 12,
    "reclaimableBytes": 1048576
  }
}
```

失败时返回非零退出码，并只输出 JSON：

```json
{
  "ok": false,
  "error": {
    "code": "invalid_target",
    "message": "target not found"
  }
}
```

## 自定义规则

默认加载两处规则：

- `~/.rcleaner/rules.toml`
- `rules/cleaner-rules.toml`

仓库内自带示例：[rules/cleaner-rules.toml](rules/cleaner-rules.toml)

规则适合描述这些场景：

- Rust/Tauri 的 `target`
- 前端的 `dist`、`node_modules`
- coverage、临时日志、工具缓存
- 某类项目的专属清理目标

## 给 AI / 自动化工具的建议

- 先调用 `rcleaner capabilities --json` 读取命令能力。
- 执行清理前必须先调用 `clean --dry-run --json`。
- AI 不应自行构造任意路径删除命令，应从 `scan --json` 返回的 target id 中选择。
- 默认不要使用 `--allow-readonly`；只有用户明确确认风险时再加。
- 解析 `ok`、`command`、`data`、`error` 字段，不要依赖本地化提示文本。

## 开发命令

```bash
npm run web:build      # 前端生产构建
npm run rust-check     # Rust 类型检查
npm run rust-test      # Rust 测试
npm run size           # 查看构建缓存和产物体积
npm run clean          # 清理构建产物
npm run clean:all      # 清理构建产物和 node_modules
```

## 数据与隐私

- 当前版本不启用 SQLite。
- 不上传扫描结果。
- 不包含远程服务、账号或 token 配置。
- 清理动作只发生在本机，并以 CLI/GUI 中明确展示的目标为准。

## 项目结构

```text
src/               React 前端
src-tauri/         Tauri 壳、Rust CLI、扫描和清理 core
rules/             示例清理规则
scripts/           dev/build 启动脚本
```

## 许可证

MIT，见 [LICENSE](LICENSE)。
