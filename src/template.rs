use std::fs;
use std::path::Path;

use crate::config::AppContext;
use crate::errors::{HarmonyAppError, Result};

#[derive(Clone, Debug)]
pub struct TemplateContext {
    pub app_name: String,
    pub bundle_name: String,
    pub version_name: String,
    pub version_code: u32,
    pub module_name: String,
    pub sdk_api_version: String,
    pub sdk_display_version: String,
    pub abi: String,
    pub rust_lib_name: String,
    pub uses_winit_ohos: bool,
    pub hvigor_package_path: String,
    pub hvigor_plugin_package_path: String,
}

#[derive(Clone, Debug)]
pub struct GeneratedFile {
    pub relative_path: &'static str,
    pub contents: String,
}

pub fn template_context(app: &AppContext) -> TemplateContext {
    TemplateContext {
        app_name: app.config.app_name.clone(),
        bundle_name: app.config.bundle_name.clone(),
        version_name: app.config.version_name.clone(),
        version_code: app.config.version_code,
        module_name: app.config.module_name.clone(),
        sdk_api_version: app.sdk.version.clone(),
        sdk_display_version: app.sdk.display_version.clone(),
        abi: app.config.abi.clone(),
        rust_lib_name: app.project.lib_name.clone(),
        uses_winit_ohos: app.project.uses_winit_ohos,
        hvigor_package_path: path_for_package_json(&app.hvigor.hvigor_package_dir),
        hvigor_plugin_package_path: path_for_package_json(&app.hvigor.hvigor_plugin_package_dir),
    }
}

pub fn generated_files(context: &TemplateContext) -> Vec<GeneratedFile> {
    let overrides = if context.uses_winit_ohos {
        Some(WINIT_TEMPLATE_OVERRIDES)
    } else {
        None
    };

    TEMPLATE_FILES
        .iter()
        .map(|(relative_path, template)| GeneratedFile {
            relative_path,
            contents: render_template(
                resolve_template(relative_path, template, overrides),
                context,
            ),
        })
        .collect()
}

pub fn write_shell_project(app: &AppContext) -> Result<()> {
    let context = template_context(app);
    let output_dir = &app.config.output_dir;
    fs::create_dir_all(output_dir).map_err(|source| HarmonyAppError::io(output_dir, source))?;

    for file in generated_files(&context) {
        let target_path = output_dir.join(file.relative_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|source| HarmonyAppError::io(parent, source))?;
        }
        fs::write(&target_path, file.contents)
            .map_err(|source| HarmonyAppError::io(&target_path, source))?;
    }
    let app_icon_bytes = if let Some(icon_path) = app.config.app_icon_path.as_ref() {
        fs::read(icon_path).map_err(|source| HarmonyAppError::io(icon_path, source))?
    } else {
        ICON_PNG_BYTES.to_vec()
    };
    let start_icon_bytes = if let Some(icon_path) = app.config.start_icon_path.as_ref() {
        fs::read(icon_path).map_err(|source| HarmonyAppError::io(icon_path, source))?
    } else {
        app_icon_bytes.clone()
    };
    write_binary_asset(
        &output_dir.join("AppScope/resources/base/media/background.png"),
        &app_icon_bytes,
    )?;
    write_binary_asset(
        &output_dir.join("AppScope/resources/base/media/foreground.png"),
        &app_icon_bytes,
    )?;
    write_binary_asset(
        &output_dir.join("entry/src/main/resources/base/media/startIcon.png"),
        &start_icon_bytes,
    )?;

    let libs_dir = output_dir
        .join("entry")
        .join("src")
        .join("main")
        .join("cpp")
        .join("libs")
        .join(&context.abi);
    fs::create_dir_all(&libs_dir).map_err(|source| HarmonyAppError::io(&libs_dir, source))?;

    copy_wrapper(&app.hvigor.wrapper_bat, &output_dir.join("hvigorw.bat"))?;
    copy_wrapper(&app.hvigor.wrapper_js, &output_dir.join("hvigorw.js"))?;
    Ok(())
}

fn copy_wrapper(from: &Path, to: &Path) -> Result<()> {
    let contents = fs::read(from).map_err(|source| HarmonyAppError::io(from, source))?;
    fs::write(to, contents).map_err(|source| HarmonyAppError::io(to, source))
}

fn write_binary_asset(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| HarmonyAppError::io(parent, source))?;
    }
    fs::write(path, bytes).map_err(|source| HarmonyAppError::io(path, source))
}

fn path_for_package_json(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn render_template(template: &str, context: &TemplateContext) -> String {
    template
        .replace("{{APP_NAME}}", &context.app_name)
        .replace("{{BUNDLE_NAME}}", &context.bundle_name)
        .replace("{{VERSION_NAME}}", &context.version_name)
        .replace("{{VERSION_CODE}}", &context.version_code.to_string())
        .replace("{{MODULE_NAME}}", &context.module_name)
        .replace("{{SDK_API_VERSION}}", &context.sdk_api_version)
        .replace("{{SDK_DISPLAY_VERSION}}", &context.sdk_display_version)
        .replace("{{ABI}}", &context.abi)
        .replace("{{RUST_LIB_NAME}}", &context.rust_lib_name)
        .replace("{{HVIGOR_PACKAGE_PATH}}", &context.hvigor_package_path)
        .replace(
            "{{HVIGOR_PLUGIN_PACKAGE_PATH}}",
            &context.hvigor_plugin_package_path,
        )
}

fn resolve_template<'a>(
    relative_path: &str,
    fallback: &'a str,
    overrides: Option<&'a [(&'a str, &'a str)]>,
) -> &'a str {
    overrides
        .and_then(|entries| entries.iter().find(|(path, _)| *path == relative_path))
        .map(|(_, template)| *template)
        .unwrap_or(fallback)
}

const TEMPLATE_FILES: &[(&str, &str)] = &[
    (
        "AppScope/app.json5",
        r#"{
  "app": {
    "bundleName": "{{BUNDLE_NAME}}",
    "vendor": "example",
    "versionCode": {{VERSION_CODE}},
    "versionName": "{{VERSION_NAME}}",
    "icon": "$media:layered_image",
    "label": "$string:app_name"
  }
}
"#,
    ),
    (
        "AppScope/resources/base/element/string.json",
        r#"{
  "string": [
    {
      "name": "app_name",
      "value": "{{APP_NAME}}"
    }
  ]
}
"#,
    ),
    (
        "AppScope/resources/base/media/layered_image.json",
        r#"{
  "layered-image": {
    "background": "$media:background",
    "foreground": "$media:foreground"
  }
}
"#,
    ),
    (
        "build-profile.json5",
        r#"{
  "app": {
    "signingConfigs": [],
    "products": [
      {
        "name": "default",
        "signingConfig": "default",
        "compileSdkVersion": {{SDK_API_VERSION}},
        "compatibleSdkVersion": {{SDK_API_VERSION}},
        "targetSdkVersion": {{SDK_API_VERSION}},
        "runtimeOS": "OpenHarmony"
      }
    ],
    "buildModeSet": [
      {
        "name": "debug"
      },
      {
        "name": "release"
      }
    ]
  },
  "modules": [
    {
      "name": "{{MODULE_NAME}}",
      "srcPath": "./entry",
      "targets": [
        {
          "name": "default",
          "applyToProducts": [
            "default"
          ]
        }
      ]
    }
  ]
}
"#,
    ),
    (
        "hvigor/hvigor-config.json5",
        r#"{
  "modelVersion": "5.0.0",
  "dependencies": {},
  "execution": {
    "daemon": false
  }
}
"#,
    ),
    (
        "hvigorfile.ts",
        r#"import { appTasks } from '@ohos/hvigor-ohos-plugin';

export default {
  system: appTasks,
  plugins: []
}
"#,
    ),
    (
        "oh-package.json5",
        r#"{
  "modelVersion": "5.0.0",
  "description": "Generated by cargo-ohos-app",
  "dependencies": {},
  "devDependencies": {}
}
"#,
    ),
    (
        "oh-package-lock.json5",
        r#"{
  "meta": {
    "stableOrder": true
  },
  "lockfileVersion": 3,
  "ATTENTION": "THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.",
  "specifiers": {},
  "packages": {}
}
"#,
    ),
    (
        "package.json",
        r#"{
  "name": "{{APP_NAME}}-ohos-app",
  "version": "1.0.0",
  "private": true,
  "description": "Generated by cargo-ohos-app",
  "devDependencies": {
    "@ohos/hvigor": "file:{{HVIGOR_PACKAGE_PATH}}",
    "@ohos/hvigor-ohos-plugin": "file:{{HVIGOR_PLUGIN_PACKAGE_PATH}}"
  }
}
"#,
    ),
    (
        "code-linter.json5",
        r#"{
  "files": [
    "**/*.ets"
  ],
  "ignore": [
    "**/node_modules/**/*",
    "**/oh_modules/**/*",
    "**/build/**/*",
    "**/.preview/**/*"
  ],
  "ruleSet": [
    "plugin:@typescript-eslint/recommended"
  ]
}
"#,
    ),
    (
        "entry/build-profile.json5",
        r#"{
  "apiType": "stageMode",
  "buildOption": {
    "externalNativeOptions": {
      "path": "./src/main/cpp/CMakeLists.txt",
      "arguments": "",
      "cppFlags": "",
      "abiFilters": [
        "{{ABI}}"
      ]
    }
  },
  "buildOptionSet": [
    {
      "name": "release",
      "arkOptions": {
        "obfuscation": {
          "ruleOptions": {
            "enable": false,
            "files": [
              "./obfuscation-rules.txt"
            ]
          }
        }
      }
    }
  ],
  "targets": [
    {
      "name": "default"
    }
  ]
}
"#,
    ),
    (
        "entry/hvigorfile.ts",
        r#"import { hapTasks } from '@ohos/hvigor-ohos-plugin';

export default {
  system: hapTasks,
  plugins: []
}
"#,
    ),
    (
        "entry/obfuscation-rules.txt",
        "# Generated by cargo-ohos-app.\n",
    ),
    (
        "entry/oh-package.json5",
        r#"{
  "name": "{{MODULE_NAME}}",
  "version": "1.0.0",
  "description": "OHOS shell generated by cargo-ohos-app",
  "main": "",
  "author": "",
  "license": "",
  "dependencies": {
    "libentry.so": "file:./src/main/cpp/types/libentry"
  }
}
"#,
    ),
    (
        "entry/src/main/module.json5",
        r#"{
  "module": {
    "name": "{{MODULE_NAME}}",
    "type": "entry",
    "description": "$string:module_desc",
    "mainElement": "EntryAbility",
    "deviceTypes": [
      "default"
    ],
    "deliveryWithInstall": true,
    "installationFree": false,
    "pages": "$profile:main_pages",
    "abilities": [
      {
        "name": "EntryAbility",
        "srcEntry": "./ets/entryability/EntryAbility.ets",
        "description": "$string:EntryAbility_desc",
        "label": "$string:EntryAbility_label",
        "startWindowIcon": "$media:startIcon",
        "startWindowBackground": "$color:start_window_background",
        "exported": true,
        "skills": [
          {
            "entities": [
              "entity.system.home"
            ],
            "actions": [
              "action.system.home"
            ]
          }
        ]
      }
    ]
  }
}
"#,
    ),
    (
        "entry/src/main/ets/entryability/EntryAbility.ets",
        r#"import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';

const DOMAIN = 0x0000;

export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(DOMAIN, 'cargo-ohos-app', '%{public}s', 'Ability onCreate');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    try {
      const mainWindow = windowStage.getMainWindowSync();
      mainWindow.setWindowLayoutFullScreen(true)
        .then(() => mainWindow.setWindowSystemBarEnable([]))
        .catch((err: Error) => {
          hilog.error(
            DOMAIN,
            'cargo-ohos-app',
            'Failed to configure immersive window: %{public}s',
            JSON.stringify(err)
          );
        });
    } catch (err) {
      hilog.error(
        DOMAIN,
        'cargo-ohos-app',
        'Failed to get main window: %{public}s',
        JSON.stringify(err)
      );
    }

    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(DOMAIN, 'cargo-ohos-app', 'Failed to load page: %{public}s', JSON.stringify(err));
      }
    });
  }
}
"#,
    ),
    (
        "entry/src/main/ets/pages/Index.ets",
        r#"import common from '@ohos.app.ability.common';
import bridge from 'libentry.so';

const runtimeBridge = bridge ?? {
  getMessage: (): string => 'Native bridge is unavailable.',
  incrementCounter: (): number => 0
};

@Entry
@Component
struct Index {
  @State message: string = runtimeBridge.getMessage();
  @State counter: number = 0;

  build() {
    Column({ space: 16 }) {
      Text('Rust -> OHOS')
        .fontSize(28)
        .fontWeight(FontWeight.Bold)

      Text(this.message)
        .fontSize(20)

      Text('Counter: ' + this.counter)
        .fontSize(18)

      Button('Call Rust')
        .onClick(() => {
          this.counter = runtimeBridge.incrementCounter();
          this.message = runtimeBridge.getMessage();
        })
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
    .alignItems(HorizontalAlign.Center)
    .padding(24)
  }
}
"#,
    ),
    (
        "entry/src/main/resources/base/profile/main_pages.json",
        r#"{
  "src": [
    "pages/Index"
  ]
}
"#,
    ),
    (
        "entry/src/main/resources/base/element/string.json",
        r#"{
  "string": [
    {
      "name": "module_desc",
      "value": "Rust generated OHOS module"
    },
    {
      "name": "EntryAbility_desc",
      "value": "Entry ability"
    },
    {
      "name": "EntryAbility_label",
      "value": "{{APP_NAME}}"
    }
  ]
}
"#,
    ),
    (
        "entry/src/main/resources/base/element/float.json",
        r#"{
  "float": [
    {
      "name": "page_text_font_size",
      "value": "18fp"
    }
  ]
}
"#,
    ),
    (
        "entry/src/main/resources/base/element/color.json",
        r##"{
  "color": [
    {
      "name": "start_window_background",
      "value": "#FFFFFF"
    }
  ]
}
"##,
    ),
    (
        "entry/src/main/cpp/CMakeLists.txt",
        r#"cmake_minimum_required(VERSION 3.5.0)
project(ohos_app_bridge)

set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})

add_library(rust_bridge STATIC IMPORTED)
set_target_properties(rust_bridge PROPERTIES
    IMPORTED_LOCATION ${NATIVERENDER_ROOT_PATH}/libs/{{ABI}}/lib{{RUST_LIB_NAME}}.a
)

add_library(entry SHARED napi_init.cpp)
set_target_properties(entry PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/out
)
target_link_libraries(entry PUBLIC libace_napi.z.so rust_bridge)
"#,
    ),
    (
        "entry/src/main/cpp/napi_init.cpp",
        r#"#include <napi/native_api.h>
#include <stdint.h>

extern "C" {
const char* ohos_app_get_message();
uint32_t ohos_app_increment_counter();
}

static napi_value GetMessage(napi_env env, napi_callback_info info)
{
    const char* message = ohos_app_get_message();
    napi_value result = nullptr;
    napi_create_string_utf8(env, message, NAPI_AUTO_LENGTH, &result);
    return result;
}

static napi_value IncrementCounter(napi_env env, napi_callback_info info)
{
    napi_value result = nullptr;
    napi_create_uint32(env, ohos_app_increment_counter(), &result);
    return result;
}

static napi_value Init(napi_env env, napi_value exports)
{
    napi_property_descriptor descriptors[] = {
        { "getMessage", nullptr, GetMessage, nullptr, nullptr, nullptr, napi_default, nullptr },
        { "incrementCounter", nullptr, IncrementCounter, nullptr, nullptr, nullptr, napi_default, nullptr }
    };
    napi_define_properties(env, exports, sizeof(descriptors) / sizeof(descriptors[0]), descriptors);
    return exports;
}

static napi_module cargoOhosAppModule = {
    .nm_version = 1,
    .nm_flags = 0,
    .nm_filename = nullptr,
    .nm_register_func = Init,
    .nm_modname = "entry",
    .nm_priv = nullptr,
    .reserved = { 0 }
};

extern "C" __attribute__((constructor)) void RegisterCargoOhosAppModule(void)
{
    napi_module_register(&cargoOhosAppModule);
}
"#,
    ),
    (
        "entry/src/main/cpp/types/libentry/index.d.ts",
        r#"declare const bridge: {
  getMessage(): string;
  incrementCounter(): number;
};

export default bridge;
"#,
    ),
    (
        "entry/src/main/cpp/types/libentry/oh-package.json5",
        r#"{
  "name": "libentry.so",
  "version": "1.0.0",
  "description": "Type definitions for the generated native bridge",
  "types": "./index.d.ts"
}
"#,
    ),
];

const WINIT_TEMPLATE_OVERRIDES: &[(&str, &str)] = &[
    (
        "entry/src/main/ets/entryability/EntryAbility.ets",
        r#"import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { window } from '@kit.ArkUI';

const DOMAIN = 0x3433;

export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(DOMAIN, 'cargo-ohos-app', '%{public}s', 'Ability onCreate');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    try {
      const mainWindow = windowStage.getMainWindowSync();
      mainWindow.setWindowLayoutFullScreen(true)
        .then(() => mainWindow.setWindowSystemBarEnable([]))
        .catch((err: Error) => {
          hilog.error(
            DOMAIN,
            'cargo-ohos-app',
            'Failed to configure immersive window: %{public}s',
            JSON.stringify(err)
          );
        });
    } catch (err) {
      hilog.error(
        DOMAIN,
        'cargo-ohos-app',
        'Failed to get main window: %{public}s',
        JSON.stringify(err)
      );
    }

    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(DOMAIN, 'cargo-ohos-app', 'Failed to load page: %{public}s', JSON.stringify(err));
      }
    });
  }
}
"#,
    ),
    (
        "entry/src/main/ets/pages/Index.ets",
        r#"import common from '@ohos.app.ability.common';
import bridge from 'libentry.so';

@Entry
@Component
struct Index {
  aboutToAppear(): void {
    const hostContext = this.getUIContext().getHostContext() as common.UIAbilityContext | undefined;
    const fontScale = hostContext?.config?.fontSizeScale;
    bridge.setFontScale(
      typeof fontScale === 'number' && Number.isFinite(fontScale) && fontScale > 0 ? fontScale : 1.0
    );
  }

  build() {
    Stack() {
      XComponent({
        id: 'winit-surface',
        type: 'surface',
        libraryname: 'entry'
      })
        .width('100%')
        .height('100%')
        .focusable(true)
        .expandSafeArea([SafeAreaType.SYSTEM], [SafeAreaEdge.TOP, SafeAreaEdge.BOTTOM])
    }
    .width('100%')
    .height('100%')
    .expandSafeArea([SafeAreaType.SYSTEM], [SafeAreaEdge.TOP, SafeAreaEdge.BOTTOM])
  }
}
"#,
    ),
    (
        "entry/src/main/cpp/CMakeLists.txt",
        r#"cmake_minimum_required(VERSION 3.5.0)
project(ohos_app_winit_shell)

set(NATIVERENDER_ROOT_PATH ${CMAKE_CURRENT_SOURCE_DIR})

add_library(rust_bridge STATIC IMPORTED)
set_target_properties(rust_bridge PROPERTIES
    IMPORTED_LOCATION ${NATIVERENDER_ROOT_PATH}/libs/{{ABI}}/lib{{RUST_LIB_NAME}}.a
)

find_library(ACE_NDK_LIB ace_ndk.z)
find_library(ACE_NAPI_LIB ace_napi.z)
find_library(UV_LIB uv)

add_library(entry SHARED napi_init.cpp)
set_target_properties(entry PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/out
)
target_link_libraries(entry PUBLIC
    rust_bridge
    ${ACE_NDK_LIB}
    ${ACE_NAPI_LIB}
    ${UV_LIB}
    libnative_window.so
)
"#,
    ),
    (
        "entry/src/main/cpp/napi_init.cpp",
        r#"#include <napi/native_api.h>
#include <stddef.h>
#include <stdint.h>

#include <cmath>
#include <mutex>

#include <ace/xcomponent/native_interface_xcomponent.h>

extern "C" {
void* ohos_winit_runtime_new();
void ohos_winit_runtime_free(void* runtime);
void ohos_winit_runtime_surface_created(
    const void* runtime,
    void* xcomponent,
    void* native_window,
    uint32_t width,
    uint32_t height,
    double scale_factor,
    double font_scale
);
void ohos_winit_runtime_surface_changed(
    const void* runtime,
    void* xcomponent,
    void* native_window,
    uint32_t width,
    uint32_t height,
    double scale_factor,
    double font_scale
);
void ohos_winit_runtime_surface_destroyed(const void* runtime);
void ohos_winit_runtime_focus(const void* runtime, bool focused);
void ohos_winit_runtime_visibility(const void* runtime, bool visible);
void ohos_winit_runtime_low_memory(const void* runtime);
void ohos_winit_runtime_frame(const void* runtime);
void ohos_winit_runtime_key(
    const void* runtime,
    uint32_t action,
    uint32_t key_code,
    bool repeat,
    int64_t device_id
);
void ohos_winit_runtime_touch(
    const void* runtime,
    uint32_t action,
    uint32_t source,
    uint64_t finger_id,
    double x,
    double y,
    double force,
    bool has_force,
    int64_t device_id,
    bool primary
);
void ohos_winit_runtime_mouse(
    const void* runtime,
    uint32_t action,
    uint32_t button,
    bool has_button,
    double x,
    double y,
    double delta_x,
    double delta_y,
    int64_t device_id,
    bool primary
);
}

namespace {

std::mutex g_runtime_mutex;
void* g_runtime = nullptr;
float g_last_mouse_x = 0.0f;
float g_last_mouse_y = 0.0f;
double g_font_scale = 1.0;
OH_NativeXComponent* g_last_xcomponent = nullptr;
void* g_last_native_window = nullptr;
uint32_t g_last_surface_width = 0;
uint32_t g_last_surface_height = 0;

void* EnsureRuntime()
{
    std::lock_guard<std::mutex> lock(g_runtime_mutex);
    if (g_runtime == nullptr) {
        g_runtime = ohos_winit_runtime_new();
    }
    return g_runtime;
}

uint32_t MapTouchAction(OH_NativeXComponent_TouchEventType type)
{
    switch (type) {
        case OH_NATIVEXCOMPONENT_DOWN:
            return 0;
        case OH_NATIVEXCOMPONENT_UP:
            return 1;
        case OH_NATIVEXCOMPONENT_MOVE:
            return 2;
        case OH_NATIVEXCOMPONENT_CANCEL:
        default:
            return 3;
    }
}

uint32_t MapPointerSource(OH_NativeXComponent_EventSourceType source)
{
    switch (source) {
        case OH_NATIVEXCOMPONENT_SOURCE_TYPE_TOUCHSCREEN:
            return 0;
        case OH_NATIVEXCOMPONENT_SOURCE_TYPE_MOUSE:
            return 1;
        case OH_NATIVEXCOMPONENT_SOURCE_TYPE_TOUCHPAD:
            return 2;
        default:
            return 3;
    }
}

uint32_t MapMouseAction(OH_NativeXComponent_MouseEventAction action)
{
    switch (action) {
        case OH_NATIVEXCOMPONENT_MOUSE_PRESS:
            return 1;
        case OH_NATIVEXCOMPONENT_MOUSE_RELEASE:
            return 2;
        case OH_NATIVEXCOMPONENT_MOUSE_MOVE:
            return 0;
        case OH_NATIVEXCOMPONENT_MOUSE_CANCEL:
            return 6;
        case OH_NATIVEXCOMPONENT_MOUSE_NONE:
        default:
            return 0;
    }
}

uint32_t MapMouseButton(OH_NativeXComponent_MouseEventButton button, bool* hasButton)
{
    *hasButton = true;
    switch (button) {
        case OH_NATIVEXCOMPONENT_LEFT_BUTTON:
            return 0;
        case OH_NATIVEXCOMPONENT_MIDDLE_BUTTON:
            return 1;
        case OH_NATIVEXCOMPONENT_RIGHT_BUTTON:
            return 2;
        case OH_NATIVEXCOMPONENT_BACK_BUTTON:
            return 3;
        case OH_NATIVEXCOMPONENT_FORWARD_BUTTON:
            return 4;
        case OH_NATIVEXCOMPONENT_NONE_BUTTON:
        default:
            *hasButton = false;
            return 0;
    }
}

uint32_t MapKeyAction(OH_NativeXComponent_KeyAction action)
{
    switch (action) {
        case OH_NATIVEXCOMPONENT_KEY_ACTION_DOWN:
            return 0;
        case OH_NATIVEXCOMPONENT_KEY_ACTION_UP:
            return 1;
        case OH_NATIVEXCOMPONENT_KEY_ACTION_UNKNOWN:
        default:
            return 2;
    }
}

double NormalizeFontScale(double fontScale)
{
    if (std::isfinite(fontScale) && fontScale > 0.0) {
        return fontScale;
    }
    return 1.0;
}

void CacheSurfaceState(OH_NativeXComponent* component, void* window, uint32_t width, uint32_t height)
{
    g_last_xcomponent = component;
    g_last_native_window = window;
    g_last_surface_width = width;
    g_last_surface_height = height;
}

void OnSurfaceCreated(OH_NativeXComponent* component, void* window)
{
    uint64_t width = 0;
    uint64_t height = 0;
    OH_NativeXComponent_GetXComponentSize(component, window, &width, &height);
    CacheSurfaceState(component, window, static_cast<uint32_t>(width), static_cast<uint32_t>(height));
    ohos_winit_runtime_surface_created(
        EnsureRuntime(),
        component,
        window,
        static_cast<uint32_t>(width),
        static_cast<uint32_t>(height),
        1.0,
        g_font_scale);
}

void OnSurfaceChanged(OH_NativeXComponent* component, void* window)
{
    uint64_t width = 0;
    uint64_t height = 0;
    OH_NativeXComponent_GetXComponentSize(component, window, &width, &height);
    CacheSurfaceState(component, window, static_cast<uint32_t>(width), static_cast<uint32_t>(height));
    ohos_winit_runtime_surface_changed(
        EnsureRuntime(),
        component,
        window,
        static_cast<uint32_t>(width),
        static_cast<uint32_t>(height),
        1.0,
        g_font_scale);
}

void OnSurfaceDestroyed(OH_NativeXComponent* component, void* window)
{
    (void)component;
    (void)window;
    g_last_xcomponent = nullptr;
    g_last_native_window = nullptr;
    g_last_surface_width = 0;
    g_last_surface_height = 0;
    ohos_winit_runtime_surface_destroyed(EnsureRuntime());
}

void OnTouch(OH_NativeXComponent* component, void* window)
{
    OH_NativeXComponent_TouchEvent touchEvent {};
    if (OH_NativeXComponent_GetTouchEvent(component, window, &touchEvent) != 0) {
        return;
    }

    OH_NativeXComponent_EventSourceType source = OH_NATIVEXCOMPONENT_SOURCE_TYPE_TOUCHSCREEN;
    OH_NativeXComponent_GetTouchEventSourceType(component, touchEvent.id, &source);

    bool primary = touchEvent.numPoints == 0 || touchEvent.touchPoints[0].id == touchEvent.id;
    ohos_winit_runtime_touch(
        EnsureRuntime(),
        MapTouchAction(touchEvent.type),
        MapPointerSource(source),
        static_cast<uint64_t>(touchEvent.id),
        touchEvent.x,
        touchEvent.y,
        touchEvent.force,
        true,
        touchEvent.deviceId,
        primary);
}

void OnMouse(OH_NativeXComponent* component, void* window)
{
    OH_NativeXComponent_MouseEvent mouseEvent {};
    if (OH_NativeXComponent_GetMouseEvent(component, window, &mouseEvent) != 0) {
        return;
    }

    g_last_mouse_x = mouseEvent.x;
    g_last_mouse_y = mouseEvent.y;

    bool hasButton = false;
    uint32_t button = MapMouseButton(mouseEvent.button, &hasButton);
    ohos_winit_runtime_mouse(
        EnsureRuntime(),
        MapMouseAction(mouseEvent.action),
        button,
        hasButton,
        mouseEvent.x,
        mouseEvent.y,
        0.0,
        0.0,
        0,
        true);
}

void OnHover(OH_NativeXComponent* component, bool isHover)
{
    (void)component;
    ohos_winit_runtime_mouse(
        EnsureRuntime(),
        isHover ? 4 : 5,
        0,
        false,
        g_last_mouse_x,
        g_last_mouse_y,
        0.0,
        0.0,
        0,
        true);
}

void OnFocus(OH_NativeXComponent* component, void* window)
{
    (void)component;
    (void)window;
    ohos_winit_runtime_focus(EnsureRuntime(), true);
}

void OnBlur(OH_NativeXComponent* component, void* window)
{
    (void)component;
    (void)window;
    ohos_winit_runtime_focus(EnsureRuntime(), false);
}

void OnKey(OH_NativeXComponent* component, void* window)
{
    (void)window;
    OH_NativeXComponent_KeyEvent* keyEvent = nullptr;
    if (OH_NativeXComponent_GetKeyEvent(component, &keyEvent) != 0 || keyEvent == nullptr) {
        return;
    }

    OH_NativeXComponent_KeyAction action = OH_NATIVEXCOMPONENT_KEY_ACTION_UNKNOWN;
    OH_NativeXComponent_KeyCode code = static_cast<OH_NativeXComponent_KeyCode>(KEY_UNKNOWN);
    int64_t deviceId = 0;
    OH_NativeXComponent_GetKeyEventAction(keyEvent, &action);
    OH_NativeXComponent_GetKeyEventCode(keyEvent, &code);
    OH_NativeXComponent_GetKeyEventDeviceId(keyEvent, &deviceId);

    ohos_winit_runtime_key(
        EnsureRuntime(),
        MapKeyAction(action),
        static_cast<uint32_t>(code),
        false,
        deviceId);
}

void OnSurfaceShow(OH_NativeXComponent* component, void* window)
{
    (void)component;
    (void)window;
    ohos_winit_runtime_visibility(EnsureRuntime(), true);
}

void OnSurfaceHide(OH_NativeXComponent* component, void* window)
{
    (void)component;
    (void)window;
    ohos_winit_runtime_visibility(EnsureRuntime(), false);
}

void OnFrame(OH_NativeXComponent* component, uint64_t timestamp, uint64_t targetTimestamp)
{
    (void)component;
    (void)timestamp;
    (void)targetTimestamp;
    ohos_winit_runtime_frame(EnsureRuntime());
}

void RegisterXComponentCallbacks(napi_env env, napi_value exports)
{
    napi_value exportInstance = nullptr;
    if (napi_get_named_property(env, exports, OH_NATIVE_XCOMPONENT_OBJ, &exportInstance) != napi_ok) {
        return;
    }

    OH_NativeXComponent* nativeXComponent = nullptr;
    if (napi_unwrap(env, exportInstance, reinterpret_cast<void**>(&nativeXComponent)) != napi_ok ||
        nativeXComponent == nullptr) {
        return;
    }

    static OH_NativeXComponent_Callback componentCallbacks {
        .OnSurfaceCreated = OnSurfaceCreated,
        .OnSurfaceChanged = OnSurfaceChanged,
        .OnSurfaceDestroyed = OnSurfaceDestroyed,
        .DispatchTouchEvent = OnTouch,
    };

    static OH_NativeXComponent_MouseEvent_Callback mouseCallbacks {
        .DispatchMouseEvent = OnMouse,
        .DispatchHoverEvent = OnHover,
    };

    OH_NativeXComponent_RegisterCallback(nativeXComponent, &componentCallbacks);
    OH_NativeXComponent_RegisterMouseEventCallback(nativeXComponent, &mouseCallbacks);
    OH_NativeXComponent_RegisterFocusEventCallback(nativeXComponent, OnFocus);
    OH_NativeXComponent_RegisterBlurEventCallback(nativeXComponent, OnBlur);
    OH_NativeXComponent_RegisterKeyEventCallback(nativeXComponent, OnKey);
    OH_NativeXComponent_RegisterSurfaceShowCallback(nativeXComponent, OnSurfaceShow);
    OH_NativeXComponent_RegisterSurfaceHideCallback(nativeXComponent, OnSurfaceHide);
    OH_NativeXComponent_RegisterOnFrameCallback(nativeXComponent, OnFrame);
}

napi_value SetFontScale(napi_env env, napi_callback_info info)
{
    size_t argc = 1;
    napi_value args[1] = { nullptr };
    if (napi_get_cb_info(env, info, &argc, args, nullptr, nullptr) != napi_ok || argc != 1) {
        return nullptr;
    }

    double fontScale = 1.0;
    if (napi_get_value_double(env, args[0], &fontScale) != napi_ok) {
        return nullptr;
    }

    g_font_scale = NormalizeFontScale(fontScale);
    if (g_last_xcomponent != nullptr && g_last_native_window != nullptr) {
        ohos_winit_runtime_surface_changed(
            EnsureRuntime(),
            g_last_xcomponent,
            g_last_native_window,
            g_last_surface_width,
            g_last_surface_height,
            1.0,
            g_font_scale);
    }

    napi_value result = nullptr;
    napi_get_undefined(env, &result);
    return result;
}

napi_value Init(napi_env env, napi_value exports)
{
    napi_property_descriptor descriptors[] = {
        { "setFontScale", nullptr, SetFontScale, nullptr, nullptr, nullptr, napi_default, nullptr },
    };
    napi_define_properties(env, exports, sizeof(descriptors) / sizeof(descriptors[0]), descriptors);
    RegisterXComponentCallbacks(env, exports);
    return exports;
}

static napi_module cargoOhosAppModule = {
    .nm_version = 1,
    .nm_flags = 0,
    .nm_filename = nullptr,
    .nm_register_func = Init,
    .nm_modname = "entry",
    .nm_priv = nullptr,
    .reserved = { 0 }
};

} // namespace

extern "C" __attribute__((constructor)) void RegisterCargoOhosAppModule(void)
{
    napi_module_register(&cargoOhosAppModule);
}
"#,
    ),
    (
        "entry/src/main/cpp/types/libentry/index.d.ts",
        r#"declare const bridge: {
  setFontScale(fontScale: number): void;
};

export default bridge;
"#,
    ),
];

const ICON_PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
    0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use crate::config::ResolvedConfig;
    use crate::project::{MetadataConfig, ProjectInfo};
    use crate::sdk::{HvigorInfo, SdkInfo};

    use super::{AppContext, TemplateContext, generated_files, write_shell_project};

    #[test]
    fn renders_bundle_and_module_names() {
        let files = generated_files(&TemplateContext {
            app_name: "demo".to_string(),
            bundle_name: "com.example.demo".to_string(),
            version_name: "1.2.3".to_string(),
            version_code: 42,
            module_name: "entry".to_string(),
            sdk_api_version: "20".to_string(),
            sdk_display_version: "6.0.0(20)".to_string(),
            abi: "arm64-v8a".to_string(),
            rust_lib_name: "counter_native".to_string(),
            uses_winit_ohos: false,
            hvigor_package_path: "D:/hvigor".to_string(),
            hvigor_plugin_package_path: "D:/hvigor-ohos-plugin".to_string(),
        });
        let app_json = files
            .iter()
            .find(|file| file.relative_path == "AppScope/app.json5")
            .unwrap();
        assert!(app_json.contents.contains("com.example.demo"));
        assert!(app_json.contents.contains("$string:app_name"));
        assert!(app_json.contents.contains("\"versionCode\": 42"));
        assert!(app_json.contents.contains("\"versionName\": \"1.2.3\""));
    }

    #[test]
    fn renders_winit_shell_when_project_uses_winit_ohos() {
        let files = generated_files(&TemplateContext {
            app_name: "demo".to_string(),
            bundle_name: "com.example.demo".to_string(),
            version_name: "1.2.3".to_string(),
            version_code: 42,
            module_name: "entry".to_string(),
            sdk_api_version: "20".to_string(),
            sdk_display_version: "6.0.0(20)".to_string(),
            abi: "arm64-v8a".to_string(),
            rust_lib_name: "counter_native".to_string(),
            uses_winit_ohos: true,
            hvigor_package_path: "D:/hvigor".to_string(),
            hvigor_plugin_package_path: "D:/hvigor-ohos-plugin".to_string(),
        });

        let page = files
            .iter()
            .find(|file| file.relative_path == "entry/src/main/ets/pages/Index.ets")
            .unwrap();
        assert!(page.contents.contains("import bridge from 'libentry.so';"));
        assert!(page.contents.contains("XComponent"));
        assert!(page.contents.contains("libraryname: 'entry'"));
        assert!(page.contents.contains("fontSizeScale"));
        assert!(page.contents.contains("bridge.setFontScale"));

        let ability = files
            .iter()
            .find(|file| file.relative_path == "entry/src/main/ets/entryability/EntryAbility.ets")
            .unwrap();
        assert!(ability.contents.contains("setWindowLayoutFullScreen(true)"));
        assert!(ability.contents.contains("setWindowSystemBarEnable([])"));

        let bridge = files
            .iter()
            .find(|file| file.relative_path == "entry/src/main/cpp/napi_init.cpp")
            .unwrap();
        assert!(bridge.contents.contains("ohos_winit_runtime_new"));
        assert!(bridge.contents.contains("double font_scale"));
        assert!(bridge.contents.contains("SetFontScale"));
        assert!(
            bridge
                .contents
                .contains("OH_NativeXComponent_RegisterCallback")
        );
    }

    #[test]
    fn writes_custom_icon_when_configured() {
        let temp = TempDir::new().unwrap();
        let app_icon_path = temp.path().join("icon.png");
        let start_icon_path = temp.path().join("start-icon.png");
        let output_dir = temp.path().join("ohos-app");
        let hvigor_root = temp.path().join("hvigor");
        fs::create_dir_all(&hvigor_root).unwrap();
        fs::write(&app_icon_path, [9_u8, 8, 7, 6]).unwrap();
        fs::write(&start_icon_path, [1_u8, 2, 3, 4]).unwrap();
        fs::write(hvigor_root.join("hvigorw.bat"), "@echo off\r\n").unwrap();
        fs::write(hvigor_root.join("hvigorw.js"), "console.log('hvigor');\n").unwrap();

        let app = AppContext {
            project: ProjectInfo {
                manifest_path: temp.path().join("Cargo.toml"),
                project_dir: temp.path().to_path_buf(),
                package_name: "demo-app".to_string(),
                package_version: "0.1.0".to_string(),
                lib_name: "demo_app".to_string(),
                target_dir: temp.path().join("target"),
                uses_winit_ohos: false,
                metadata_config: MetadataConfig::default(),
            },
            config: ResolvedConfig {
                deveco_studio_dir: temp.path().join("deveco"),
                ohpm_path: temp.path().join("ohpm.bat"),
                sdk_root: temp.path().join("sdk"),
                sdk_version: Some("20".to_string()),
                version_name: "1.2.3".to_string(),
                version_code: 42,
                app_name: "Demo".to_string(),
                app_icon_path: Some(app_icon_path.clone()),
                start_icon_path: Some(start_icon_path.clone()),
                target: "aarch64-unknown-linux-ohos".to_string(),
                abi: "arm64-v8a".to_string(),
                profile_dir: "debug".to_string(),
                output_dir: output_dir.clone(),
                bundle_name: "com.example.demo".to_string(),
                module_name: "entry".to_string(),
            },
            sdk: SdkInfo {
                version: "20".to_string(),
                display_version: "6.0.0(20)".to_string(),
                root: temp.path().join("sdk"),
                version_dir: temp.path().join("sdk/20"),
                native_dir: temp.path().join("sdk/20/native"),
                toolchains_dir: temp.path().join("sdk/20/toolchains"),
            },
            hvigor: HvigorInfo {
                wrapper_bat: hvigor_root.join("hvigorw.bat"),
                wrapper_js: hvigor_root.join("hvigorw.js"),
                hvigor_package_dir: hvigor_root.clone(),
                hvigor_plugin_package_dir: hvigor_root,
            },
        };

        write_shell_project(&app).unwrap();
        assert_eq!(
            fs::read(output_dir.join("AppScope/resources/base/media/background.png")).unwrap(),
            vec![9_u8, 8, 7, 6]
        );
        assert_eq!(
            fs::read(output_dir.join("entry/src/main/resources/base/media/startIcon.png")).unwrap(),
            vec![1_u8, 2, 3, 4]
        );
    }
}
