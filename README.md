# rust-webui-kit

一套可复用的 Rust 基础模块，用来构建**界面用网页技术渲染的原生 Windows 应用**：
自带托盘图标与弹窗菜单、把宿主 API 暴露给网页、并用 WebView2 或已安装的
Chromium 承载多个窗口。它们都是纯机制，不含任何业务逻辑，供上层应用按需组合。

附带一个可直接运行的示例应用 **Orbit**：一个多用窗口的托盘小工具，用这几个模块
写成，用来演示它们如何协同工作。

- 不打包浏览器内核：设置界面用系统自带 WebView2，兜底时借用已安装的 Edge/Chrome
- 页面经本地回环（`127.0.0.1`）提供，交互全走本机，断网也能用
- 原生实现，无 GC、无运行时；单个可执行文件

## 🚀 快速开始

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;" + $env:Path
cargo run -p orbit-demo
```

托盘里会出现图标；右键弹出的菜单就是一个实时任务列表，勾选即开关对应窗口。
`ORBIT_ENGINE=browser|webview` 可强制引擎，`ORBIT_BROWSER=<exe>` 可指定借用的浏览器，
`ORBIT_DATA_DIR=...` 可换数据目录。

> GNU 工具链下 WebView2 走动态加载，运行时需要 `WebView2Loader.dll`；demo 的
> `build.rs` 会自动把它复制到可执行文件旁，`cargo run` 直接可用。

## 🧩 五个基础 crate

- **`winkit`** — 最小可复用的 Win32 辅助：进程 DPI 感知（`enable_per_monitor_dpi`）
  与主屏缩放（`dpi_scale`）、从 `.ico` 字节构造 `HICON`（`from_ico`、`set_window_icon`）。
  被其它 UI crate 依赖。
- **`traykit`** — 系统托盘图标 + 自绘 Fluent 风格弹窗菜单。菜单内容是**数据**：
  宿主每次弹出时用 `MenuItem::command / toggle / info / separator` 现拼，勾选状态实时读取；
  命令通过 `TrayConfig::on_command` 回调。含圆角、悬停、键盘导航、DPI 适配。
- **`browserhost`** — 定位、启动并驱动系统已安装的 Chromium（CDP）：独立配置目录、
  窗口形态（普通 / `--app` 无边框 / 固定尺寸）、Cookie 读写、脚本求值、按 profile 回收进程。
  找不到浏览器时返回可识别的 `NoBrowser` 错误。
- **`webmsg`** — 本地回环 HTTP 桥。网页里 `window.<命名空间>.任意方法(参数)` 即一次调用，
  `.on(事件, 回调)` 订阅宿主推送（`window.__hostDeliver`）；宿主向页面回值或广播事件。
  带 `Host`/`Origin` 回环校验，挡 DNS rebinding。
- **`websurface`** — 多窗口 Web UI 宿主。一个 `Surface` 就是一个渲染某页面的窗口，两种引擎
  （内嵌 WebView2、或借用进程外 Chromium）；`WebHost` 在一条线程上持有全部窗口并泵消息，
  其它线程可用 `WebHostHandle` 开窗、推送、关窗、置顶、查询存活窗口。推送分两种：
  `post`/`broadcast` 逐条有序投递；`broadcast_rev` 为**带版本号的合并式**（同名只保留最高
  `rev`，且绝不回退）。`rev` 由调用方在产生变更处打上，于是“最新”按**意图顺序**而非调用
  先后判定：突发只发最新一份，乱序也不会让窗口先看到新、再退回旧。

依赖方向：`websurface` → { `webmsg`、`browserhost`、`winkit` }；`traykit` → `winkit`。
两者彼此独立，可单独使用（例如只要托盘就只依赖 `traykit`）。

## 🔌 两个引擎

- **WebView2（内嵌）**：窗口是我们自己的，`websurface` 在其中嵌入系统 WebView2。
  原生支持程序化缩放、并发消息推送，并处理高 DPI 与 `WM_DPICHANGED`。
- **借用 Chromium（进程外）**：机器上没有 WebView2 时，用 `browserhost` 以 `--app`
  打开已安装的 Edge/Chrome，窗口属于浏览器进程；推送经 CDP 完成。

`Engine::Auto` 优先 WebView2，其次借用；也可显式指定。`Caps` 描述每个窗口的能力，
调用方据此降级，而不是假设两者完全对等。

## ✅ 构建与测试

```powershell
cargo check --workspace --all-targets --message-format short   # 必须零警告
cargo test --workspace
cargo check -p websurface --no-default-features               # feature 组合
cargo check -p websurface --no-default-features --features webview2
cargo check -p websurface --no-default-features --features borrowed-browser
```

`websurface` 默认开启 `webview2` 与 `borrowed-browser` 两个 feature，可按需裁剪。

## 🧭 设计原则

- **机制在此，策略在外**：crate 只做通用机制，业务规则、页面与文案由上层应用提供。
- **数据驱动 UI**：菜单行、窗口配置都以数据/回调传入，crate 不内置任何应用形状。
- **错误信息用英文**：便于复用与本地化，上层应用负责映射成面向用户的文字。
- **零依赖 WebView2 之外的重资产**：浏览器来自系统，页面来自本地回环。

## 📄 许可

[MIT](LICENSE)。
