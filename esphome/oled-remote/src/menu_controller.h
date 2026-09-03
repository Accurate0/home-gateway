#pragma once
#include "menu_entities.h"
#include "display_utils.h"

class MenuController {
public:

  static void open(int& menu_index, int app_mode, bool& menu_active) {
    if (MENU_LIST_COUNT <= 1) return;
    menu_index = 0;
    for (int i = 0; i < MENU_LIST_COUNT; i++) {
      if (MENU_LIST[i].id == app_mode) { menu_index = i; break; }
    }
    menu_active = true;
  }

  static void confirm(int& app_mode, int menu_index, bool& menu_active, bool& updated_ui) {
    app_mode    = MENU_LIST[menu_index].id;
    menu_active = false;
    updated_ui  = true;
  }

  static void prev(int& menu_index) {
    menu_index = (menu_index - 1 + MENU_LIST_COUNT) % MENU_LIST_COUNT;
  }

  static void next(int& menu_index) {
    menu_index = (menu_index + 1) % MENU_LIST_COUNT;
  }

  template<class D, class F>
  static void draw(D* it, F* icon_big, F* icon_small,
                   F* font_base, F* font_small_f, int menu_index) {
    it->clear();

    const int cur  = menu_index;
    const int prev = (cur - 1 + MENU_LIST_COUNT) % MENU_LIST_COUNT;
    const int nxt  = (cur + 1) % MENU_LIST_COUNT;

    it->print(14,  16, icon_small, COLOR_ON, display::TextAlign::CENTER, MENU_LIST[prev].icon);
    it->print(114, 16, icon_small, COLOR_ON, display::TextAlign::CENTER, MENU_LIST[nxt].icon);
    it->print(64,  14, icon_big,   COLOR_ON, display::TextAlign::CENTER, MENU_LIST[cur].icon);
    it->print(64,  40, font_base,  COLOR_ON, display::TextAlign::CENTER, MENU_LIST[cur].name);

    it->display();
  }
};
