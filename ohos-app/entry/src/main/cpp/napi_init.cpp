#include <hilog/log.h>
#include <napi/native_api.h>
#include <stdint.h>

#include <cmath>

extern "C" {
const char* ohos_app_get_message();
uint32_t ohos_app_increment_counter();
}

namespace {

constexpr unsigned int DEFAULT_LOG_DOMAIN = 0x3433;
constexpr const char* DEFAULT_LOG_TAG = "rust";
double g_density_scale = 1.0;
double g_font_scale = 1.0;

LogLevel NormalizeLogLevel(uint32_t level)
{
    switch (level) {
        case LOG_DEBUG:
            return LOG_DEBUG;
        case LOG_INFO:
            return LOG_INFO;
        case LOG_WARN:
            return LOG_WARN;
        case LOG_ERROR:
            return LOG_ERROR;
        case LOG_FATAL:
            return LOG_FATAL;
        default:
            return LOG_INFO;
    }
}

unsigned int NormalizeLogDomain(uint32_t domain)
{
    return domain <= 0xFFFF ? domain : DEFAULT_LOG_DOMAIN;
}

const char* SafeString(const char* value, const char* fallback)
{
    return value != nullptr && value[0] != '\0' ? value : fallback;
}

double NormalizePositiveValue(double value)
{
    if (std::isfinite(value) && value > 0.0) {
        return value;
    }
    return 1.0;
}

} // namespace

extern "C" int cargo_ohos_app_hilog(uint32_t level, uint32_t domain, const char* tag, const char* message)
{
    return OH_LOG_Print(
        LOG_APP,
        NormalizeLogLevel(level),
        NormalizeLogDomain(domain),
        SafeString(tag, DEFAULT_LOG_TAG),
        "%{public}s",
        SafeString(message, ""));
}

extern "C" double cargo_ohos_app_density_scale()
{
    return g_density_scale;
}

extern "C" double cargo_ohos_app_font_scale()
{
    return g_font_scale;
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

static napi_value SetScaleFactor(napi_env env, napi_callback_info info)
{
    size_t argc = 1;
    napi_value args[1] = { nullptr };
    if (napi_get_cb_info(env, info, &argc, args, nullptr, nullptr) != napi_ok || argc != 1) {
        return nullptr;
    }

    double scaleFactor = 1.0;
    if (napi_get_value_double(env, args[0], &scaleFactor) != napi_ok) {
        return nullptr;
    }

    g_density_scale = NormalizePositiveValue(scaleFactor);

    napi_value result = nullptr;
    napi_get_undefined(env, &result);
    return result;
}

static napi_value SetFontScale(napi_env env, napi_callback_info info)
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

    g_font_scale = NormalizePositiveValue(fontScale);

    napi_value result = nullptr;
    napi_get_undefined(env, &result);
    return result;
}

static napi_value Init(napi_env env, napi_value exports)
{
    napi_property_descriptor descriptors[] = {
        { "getMessage", nullptr, GetMessage, nullptr, nullptr, nullptr, napi_default, nullptr },
        { "incrementCounter", nullptr, IncrementCounter, nullptr, nullptr, nullptr, napi_default, nullptr },
        { "setScaleFactor", nullptr, SetScaleFactor, nullptr, nullptr, nullptr, napi_default, nullptr },
        { "setFontScale", nullptr, SetFontScale, nullptr, nullptr, nullptr, napi_default, nullptr }
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
