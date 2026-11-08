# 专注计时 · 低功耗桌面组件

Rust + egui 0.37 写的单窗口桌面小组件，用来验证「原生 GUI 也能做到 30MB 级内存、空闲零重绘」。

- 单文件 exe，静态链接 C 运行库，目标机无需装任何运行库
- 只依赖 `eframe`，没有 Node、没有 WebView、没有浏览器内核

## 拿到成品 exe（本地不需要安装任何工具链）

1. 在 GitHub 新建一个空仓库（建议 public，Actions 免费不限时长）
2. 把本目录推上去：

```bash
git remote add origin https://github.com/<你的账号>/<仓库名>.git
git push -u origin main
```

3. 打开仓库的 **Actions** 页面，等 `build-windows-exe` 跑完（首次约 5~8 分钟，之后有缓存约 2 分钟）
4. 在该次运行页面底部 **Artifacts** 下载 `focus-timer-windows-x64`，解压即得 `focus-timer.exe`

## 发版（可选）

打 tag 会自动构建并发 Release，得到一个固定下载地址：

```bash
git tag v0.1.0
git push --tags
```

## 本地开发（可选，需要 Rust 工具链）

Windows 需要 rustup + VS Build Tools（MSVC 提供 link.exe 与 Windows SDK）：

```bash
cargo run --release
```

打包发布版：

```bash
cargo build --release
# 产物：target/release/focus-timer.exe
```

## 目录

```
src/main.rs                     全部逻辑，约 200 行
preview.html                    等效 HTML 预览，浏览器直接打开即可交互（无需 Rust）
Cargo.toml                      依赖与体积优化配置
.github/workflows/build.yml     push 自动构建 windows exe
.github/workflows/release.yml   打 tag 自动发 Release
```

## 设计要点

**空闲不重绘**。只在计时运行时调用 `ctx.request_repaint()`；停下后不再调度任何重绘，egui 自动挂起，CPU 归零。停下的那一刻会补一帧，把「已休眠」状态画出来，然后彻底静止。

**没有循环、没有后台线程**。整个程序靠事件驱动，不用 `setInterval` 轮询。

**体积优先**。`Cargo.toml` 里 `opt-level = "z"` + `lto` + `strip` + `panic = "abort"`，预期单文件 5~8MB。

## 预期运行时数据

| 指标 | 量级 |
|---|---|
| 单文件体积 | 5~8 MB |
| 启动内存 | 30~45 MB |
| 空闲 CPU | 0%（不请求重绘） |
| 冷启动 | < 300 ms |
