#pragma once
#include <string>
#include <cstring>
#include "esphome/components/json/json_util.h"

static const int GW_MAX_LIGHTS = 8;
static const int GW_MAX_MEDIA  = 4;

static const char* GW_STATE_QUERY =
  "{\"query\":\"{entities{__typename ... on LightEntity{id name on capabilities}"
  " ... on MediaPlayerEntity{id name state}}}\"}";

class GatewayClient {
public:

  struct Light {
    std::string id;
    std::string name;
    bool        on;
    bool        dimmable;
  };

  struct Media {
    std::string id;
    std::string name;
    std::string state;
  };

  static Light lights[GW_MAX_LIGHTS];
  static Media media[GW_MAX_MEDIA];
  static int   light_count;
  static int   media_count;

  static bool ingest(const std::string& body, bool& updated_ui) {
    Light next_lights[GW_MAX_LIGHTS];
    Media next_media[GW_MAX_MEDIA];
    int   nl = 0;
    int   nm = 0;

    bool ok = esphome::json::parse_json(body, [&](JsonObject root) -> bool {
      JsonArray arr = root["data"]["entities"];
      if (arr.isNull()) return false;

      for (JsonObject e : arr) {
        const char* type = e["__typename"];
        if (type == nullptr) continue;

        if (strcmp(type, "LightEntity") == 0 && nl < GW_MAX_LIGHTS) {
          next_lights[nl].id       = e["id"].as<const char*>();
          next_lights[nl].name     = e["name"].as<const char*>();
          next_lights[nl].on       = e["on"].as<bool>();
          next_lights[nl].dimmable = false;
          JsonArray caps = e["capabilities"].as<JsonArray>();
          for (JsonVariant c : caps) {
            if (strcmp(c.as<const char*>(), "BRIGHTNESS") == 0) {
              next_lights[nl].dimmable = true;
            }
          }
          nl++;
        } else if (strcmp(type, "MediaPlayerEntity") == 0 && nm < GW_MAX_MEDIA) {
          next_media[nm].id    = e["id"].as<const char*>();
          next_media[nm].name  = e["name"].as<const char*>();
          next_media[nm].state = e["state"].isNull() ? "" : e["state"].as<const char*>();
          nm++;
        }
      }
      return true;
    });

    if (!ok || (nl == 0 && nm == 0)) return false;

    for (int i = 0; i < nl; i++) lights[i] = next_lights[i];
    for (int i = 0; i < nm; i++) media[i] = next_media[i];
    light_count = nl;
    media_count = nm;
    updated_ui  = true;
    return true;
  }

  static std::string lightToggle(int idx) {
    if (idx < 0 || idx >= light_count) return "";
    return mutation("light(id:\\\"" + lights[idx].id + "\\\"){toggle}");
  }

  static std::string lightBrightnessMove(int idx, int step) {
    if (idx < 0 || idx >= light_count) return "";
    return mutation("light(id:\\\"" + lights[idx].id + "\\\"){brightnessMove(input:{value:" +
                    std::to_string(step) + ",onOff:false})}");
  }

  static std::string mediaAction(int idx, const char* field) {
    if (idx < 0 || idx >= media_count) return "";
    return mutation("mediaPlayer(id:\\\"" + media[idx].id + "\\\"){" + field + "}");
  }

private:

  static std::string mutation(const std::string& selection) {
    return "{\"query\":\"mutation{" + selection + "}\"}";
  }
};

GatewayClient::Light GatewayClient::lights[GW_MAX_LIGHTS];
GatewayClient::Media GatewayClient::media[GW_MAX_MEDIA];
int GatewayClient::light_count = 0;
int GatewayClient::media_count = 0;
