#include "GDEP133C02.h"
#include "comms.h"
#include "image.h"
#include "pindefine.h"

void app_main(void) {

  //=============   Control Signal Setting ========================
  initialGpio();
  initialSpi();
  setGpioLevel(LOAD_SW, GPIO_HIGH);
  epdHardwareReset();
  setPinCsAll(GPIO_HIGH);
  //===============================================================

  // epdStatus = checkDriverICStatus();

  initEPD();
  epdDisplayColorBar();
  delayms(2000);

  initEPD();
  pic_display_test(gImage);
  delayms(2000);

  initEPD();
  draw_checkerboard();
  delayms(2000);

  initEPD();
  epdDisplayColor(WHITE);
  delayms(2000);
  //====================================================

  while (1)
    ;
}
