# cargo-ohos-app

`cargo-ohos-app` 是一个 Cargo 外部子命令，用来把 Rust library 项目包装成 OHOS Stage Model 工程，并串联 OHOS 打包流程。

## 能力

- `cargo ohos-app init`
- `cargo ohos-app build`
- `cargo ohos-app package`

`package` 默认产出 `.hap`，可通过 `--artifact app` 切换为 `.app`。
也支持通过 `--target arm64-v8a|armeabi-v7a|x86_64|loongarch64` 切换目标架构；例如模拟器可用 `--target x86_64`。

## Rust 侧约定

首版采用 `ArkUI 壳 + Rust 原生库 + C ABI` 路线。Rust 项目需要：

- 有 `lib` target
- 导出以下符号

打包时工具会把 Rust 库按 `staticlib` 方式编进 `libentry.so`，避免运行时再去解析额外的 Rust `.so`。如果你希望本地也直接生成静态库，推荐：

```toml
[lib]
crate-type = ["cdylib", "staticlib"]
```

```rust
#[unsafe(no_mangle)]
pub extern "C" fn ohos_app_get_message() -> *const std::ffi::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn ohos_app_increment_counter() -> u32;
```

如果项目依赖 `tgui-winit-ohos`，`cargo-ohos-app` 现在会自动切换到 `XComponent` 壳模板。
这时不需要再手写 `NativeXComponent -> Rust` 桥接代码，只需要在 Rust 侧导出标准运行时入口：

```rust
use winit_core::application::ApplicationHandler;
use winit_ohos::export_ohos_winit_app;

#[derive(Default)]
struct MyApp;

impl ApplicationHandler for MyApp {}

export_ohos_winit_app!(MyApp::default);
```

生成的壳会自动把 surface、focus、visibility、frame、touch、mouse、key 回调转发到
`tgui-winit-ohos` 的运行时桥接层。

## 快速开始

最小 C ABI 示例位于 [examples/counter-native](examples/counter-native)。

```powershell
cd examples/counter-native
cargo run -- init
cargo run -- build
cargo run -- package
cargo run -- package --target x86_64
```

推荐的 `tgui-winit-ohos` 联调与打包示例位于 [examples/winit-smoke](examples/winit-smoke)，默认面向
`x86_64-unknown-linux-ohos` 模拟器目标：

```powershell
cargo run -- package --manifest-path .\examples\winit-smoke\Cargo.toml
```

如果已经安装为 Cargo 子命令，也可以直接这样调用：

```powershell
cargo install cargo-ohos-app
cargo ohos-app package --manifest-path .\examples\winit-smoke\Cargo.toml
```

如需覆盖默认目标，例如切到设备 ABI：

```powershell
cargo ohos-app package --manifest-path .\examples\winit-smoke\Cargo.toml --target arm64-v8a
```

本地开发时也可以直接安装当前仓库：

```powershell
cargo install --path .
```

## 配置

项目配置写在 `Cargo.toml` 的 `package.metadata.ohos-app` 中，支持 `default`、`debug`、`release` 三层：

```toml
[package.metadata.ohos-app.default]
deveco_studio_dir = "D:\\Apps\\code\\DevEco Studio"
ohpm_path = "D:\\Apps\\code\\DevEco Studio\\tools\\ohpm\\bin\\ohpm.bat"
sdk_root = "C:\\Users\\your-user\\AppData\\Local\\OpenHarmony\\Sdk"
version_name = "1.0.0"
version_code = 1
app_name = "Demo App"
app_icon_path = "assets/app-icon.png"
start_icon_path = "assets/start-icon.png"
bundle_name = "com.example.demo"
module_name = "entry"
target = "arm64-v8a"
output_dir = "ohos-app"

[package.metadata.ohos-app.debug]
output_dir = "ohos-app-debug"

[package.metadata.ohos-app.release]
output_dir = "ohos-app-release"
profile = "release"
```

支持字段：

- `deveco_studio_dir`
- `ohpm_path`
- `sdk_root`
- `sdk_version`
- `version_name`
- `version_code`
- `app_name`
- `app_icon_path`
- `start_icon_path`
- `bundle_name`
- `module_name`
- `target`
- `profile`
- `output_dir`

优先级为：CLI 参数 > 环境变量 > `Cargo.toml` metadata > 内置默认值。

其中以下三项没有内置默认值，缺失时会直接报错并停止执行：

- `deveco_studio_dir`
- `ohpm_path`
- `sdk_root`
