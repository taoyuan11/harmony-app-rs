#include <napi/native_api.h>
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
