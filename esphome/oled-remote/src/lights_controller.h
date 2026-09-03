#pragma once
#include <string>
#include <cstring>
#include <cctype>
#include <cstdio>
#include "gateway_client.h"
#include "display_utils.h"

class LightsController {
public:

  static void prevLight(int& idx) {
    if (GatewayClient::light_count <= 0) { idx = 0; return; }
    idx = (idx - 1 + GatewayClient::light_count) % GatewayClient::light_count;
  }

  static void nextLight(int& idx) {
    if (GatewayClient::light_count <= 0) { idx = 0; return; }
    idx = (idx + 1) % GatewayClient::light_count;
  }

  static void clamp(int& idx) {
    if (idx < 0 || idx >= GatewayClient::light_count) idx = 0;
  }

  static void toggleLocal(int idx, bool& updated_ui) {
    if (idx < 0 || idx >= GatewayClient::light_count) return;
    GatewayClient::lights[idx].on = !GatewayClient::lights[idx].on;
    updated_ui = true;
  }

  static bool dimmable(int idx) {
    if (idx < 0 || idx >= GatewayClient::light_count) return false;
    return GatewayClient::lights[idx].dimmable;
  }

  template<class D, class F>
  static void draw(D* it, F* font_small, int selected_idx, bool& updated_ui) {
    if (!updated_ui) return;
    updated_ui = false;

    it->clear();

    const int count = GatewayClient::light_count;
    if (count == 0) {
      it->print(64, 26, font_small, COLOR_ON, display::TextAlign::CENTER, "NO LIGHTS");
      draw_bottom_menu(it, font_small, "^", "", "v");
      it->display();
      return;
    }

    static int scroll_top = 0;
    if (selected_idx < scroll_top)      scroll_top = selected_idx;
    if (selected_idx >= scroll_top + 4) scroll_top = selected_idx - 3;
    if (scroll_top > count - 4)         scroll_top = count > 4 ? count - 4 : 0;
    if (scroll_top < 0)                 scroll_top = 0;

    for (int row = 0; row < 4; row++) {
      int li = scroll_top + row;
      if (li >= count) break;

      const bool sel = (li == selected_idx);
      const bool on  = GatewayClient::lights[li].on;
      const bool dim = GatewayClient::lights[li].dimmable;

      const int y_top    = row * 13;
      const int y_center = y_top + 6;

      Color fg = COLOR_ON;
      if (sel) {
        it->filled_rectangle(0, y_top, 128, 13, COLOR_ON);
        fg = COLOR_OFF;
      }

      char buf[16];
      strncpy(buf, GatewayClient::lights[li].name.c_str(), sizeof(buf) - 1);
      buf[sizeof(buf) - 1] = '\0';
      for (int k = 0; buf[k]; k++) buf[k] = toupper((unsigned char)buf[k]);
      it->print(4, y_center, font_small, fg, display::TextAlign::CENTER_LEFT, buf);

      if (dim) {
        int tx, ty, tw, th;
        it->get_text_bounds(4, y_center, buf, font_small,
                            display::TextAlign::CENTER_LEFT, &tx, &ty, &tw, &th);
        it->filled_circle(tx + tw + 7, y_center, 2, fg);
      }

      if (on) {
        it->filled_circle(119, y_center, 4, fg);
      } else {
        it->circle(119, y_center, 4, fg);
      }
    }

    draw_bottom_menu(it, font_small, "^", "", "v");
    it->display();
  }
};
