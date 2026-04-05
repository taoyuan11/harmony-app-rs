use std::fs;
use std::path::Path;

use crate::config::AppContext;
use crate::errors::{HarmonyAppError, Result};

#[derive(Clone, Debug)]
pub struct TemplateContext {
    pub app_name: String,
    pub bundle_name: String,
    pub module_name: String,
    pub sdk_api_version: String,
    pub sdk_display_version: String,
    pub abi: String,
    pub rust_lib_name: String,
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
        app_name: app.project.package_name.clone(),
        bundle_name: app.config.bundle_name.clone(),
        module_name: app.config.module_name.clone(),
        sdk_api_version: app.sdk.version.clone(),
        sdk_display_version: app.sdk.display_version.clone(),
        abi: app.config.abi.clone(),
        rust_lib_name: app.project.lib_name.clone(),
        hvigor_package_path: path_for_package_json(&app.hvigor.hvigor_package_dir),
        hvigor_plugin_package_path: path_for_package_json(&app.hvigor.hvigor_plugin_package_dir),
    }
}

pub fn generated_files(context: &TemplateContext) -> Vec<GeneratedFile> {
    TEMPLATE_FILES
        .iter()
        .map(|(relative_path, template)| GeneratedFile {
            relative_path,
            contents: render_template(template, context),
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
    write_binary_asset(
        &output_dir.join("AppScope/resources/base/media/background.png"),
        ICON_PNG_BYTES,
    )?;
    write_binary_asset(
        &output_dir.join("AppScope/resources/base/media/foreground.png"),
        ICON_PNG_BYTES,
    )?;
    write_binary_asset(
        &output_dir.join("entry/src/main/resources/base/media/startIcon.png"),
        ICON_PNG_BYTES,
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

const TEMPLATE_FILES: &[(&str, &str)] = &[
    (
        "AppScope/app.json5",
        r#"{
  "app": {
    "bundleName": "{{BUNDLE_NAME}}",
    "vendor": "example",
    "versionCode": 1000000,
    "versionName": "1.0.0",
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
        r#"import bridge from 'libentry.so';

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

const ICON_PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
    0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[cfg(test)]
mod tests {
    use super::{TemplateContext, generated_files};

    #[test]
    fn renders_bundle_and_module_names() {
        let files = generated_files(&TemplateContext {
            app_name: "demo".to_string(),
            bundle_name: "com.example.demo".to_string(),
            module_name: "entry".to_string(),
            sdk_api_version: "20".to_string(),
            sdk_display_version: "6.0.0(20)".to_string(),
            abi: "arm64-v8a".to_string(),
            rust_lib_name: "counter_native".to_string(),
            hvigor_package_path: "D:/hvigor".to_string(),
            hvigor_plugin_package_path: "D:/hvigor-ohos-plugin".to_string(),
        });
        let app_json = files
            .iter()
            .find(|file| file.relative_path == "AppScope/app.json5")
            .unwrap();
        assert!(app_json.contents.contains("com.example.demo"));
    }
}
