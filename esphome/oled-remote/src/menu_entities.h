#pragma once

static const int APP_LIGHTS   = 0;
static const int APP_MEDIA    = 1;
static const int APP_SETTINGS = 2;

struct MenuEntry {
  int         id;
  const char* icon;
  const char* name;
};

static const MenuEntry MENU_LIST[] = {
  { APP_LIGHTS,   "\ue0f0", "LIGHTS"   },
  { APP_MEDIA,    "\ue639", "MEDIA"    },
  { APP_SETTINGS, "\ue8b8", "SETTINGS" },
};
static const int MENU_LIST_COUNT = sizeof(MENU_LIST) / sizeof(MENU_LIST[0]);
