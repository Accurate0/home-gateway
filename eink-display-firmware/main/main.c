#include "GDEP133C02.h"
#include "comms.h"
#include "image.h"
#include "pindefine.h"

static comms_t comms_obj;
static gdep133c02_t epd_obj;
static unsigned char epd_buffer[EPD_IMAGE_DATA_BUFFER_SIZE];

void app_main(void) {
  comms_init_gpio(&comms_obj);
  comms_init_spi(&comms_obj);

  gdep133c02_init(&epd_obj, &comms_obj, epd_buffer);

  comms_set_gpio_level(&comms_obj, LOAD_SW, GPIO_HIGH);
  gdep133c02_hardware_reset(&epd_obj);
  gdep133c02_set_pin_cs_all(&epd_obj, GPIO_HIGH);

  gdep133c02_init_epd(&epd_obj);
  gdep133c02_display_color_bar(&epd_obj);
  comms_delay_ms(&comms_obj, 2000);

  gdep133c02_init_epd(&epd_obj);
  gdep133c02_pic_display_test(&epd_obj, gImage);
  comms_delay_ms(&comms_obj, 2000);

  gdep133c02_init_epd(&epd_obj);
  gdep133c02_draw_checkerboard(&epd_obj);
  comms_delay_ms(&comms_obj, 2000);

  gdep133c02_init_epd(&epd_obj);
  gdep133c02_display_color(&epd_obj, EPD_WHITE);
  comms_delay_ms(&comms_obj, 2000);
  //====================================================

  while (1)
    ;
}
