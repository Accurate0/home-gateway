#pragma once
#include <string>
#include <cstring>
#include <cctype>
#include "gateway_client.h"
#include "display_utils.h"

class MediaController {
public:

  static void prevPlayer(int& idx) {
    if (GatewayClient::media_count <= 0) { idx = 0; return; }
    idx = (idx - 1 + GatewayClient::media_count) % GatewayClient::media_count;
  }

  static void nextPlayer(int& idx) {
    if (GatewayClient::media_count <= 0) { idx = 0; return; }
    idx = (idx + 1) % GatewayClient::media_count;
  }

  static void clamp(int& idx) {
    if (idx < 0 || idx >= GatewayClient::media_count) idx = 0;
  }

  template<class D, class F>
  static void draw(D* it, F* font_base, F* font_small, int selected_idx, bool& updated_ui) {
    if (!updated_ui) return;
    updated_ui = false;

    it->clear();

    const int count = GatewayClient::media_count;
    if (count == 0) {
      it->print(64, 24, font_small, COLOR_ON, display::TextAlign::CENTER, "NO PLAYERS");
      it->display();
      return;
    }

    if (selected_idx >= count) selected_idx = 0;

    char name[18];
    strncpy(name, GatewayClient::media[selected_idx].name.c_str(), sizeof(name) - 1);
    name[sizeof(name) - 1] = '\0';
    for (int k = 0; name[k]; k++) name[k] = toupper((unsigned char)name[k]);

    char state[18];
    strncpy(state, GatewayClient::media[selected_idx].state.c_str(), sizeof(state) - 1);
    state[sizeof(state) - 1] = '\0';
    for (int k = 0; state[k]; k++) state[k] = toupper((unsigned char)state[k]);

    it->print(64, 10, font_small, COLOR_ON, display::TextAlign::CENTER, name);
    it->rectangle(0, 20, 128, 1);
    it->print(64, 36, font_base, COLOR_ON, display::TextAlign::CENTER, state);

    if (count > 1) {
      char pos[16];
      snprintf(pos, sizeof(pos), "%d/%d",
               (selected_idx + 1) % 100, count % 100);
      it->print(124, 10, font_small, COLOR_ON, display::TextAlign::CENTER_RIGHT, pos);
    }

    it->display();
  }
};
