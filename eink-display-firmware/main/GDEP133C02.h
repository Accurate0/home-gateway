
#ifndef __GDEP133C02_H__
#define __GDEP133C02_H__

#include "comms.h"

#define EPD_BLACK 0x00
#define EPD_WHITE 0x11
#define EPD_YELLOW 0x22
#define EPD_RED 0x33
#define EPD_BLUE 0x55
#define EPD_GREEN 0x66

#define EPD_PSR 0x00
#define EPD_PWR 0x01
#define EPD_POF 0x02
#define EPD_PON 0x04
#define EPD_BTST_N 0x05
#define EPD_BTST_P 0x06
#define EPD_DTM 0x10
#define EPD_DRF 0x12
#define EPD_CDI 0x50
#define EPD_TCON 0x60
#define EPD_TRES 0x61
#define EPD_PTLW 0x83
#define EPD_AN_TM 0x74
#define EPD_AGID 0x86
#define EPD_BUCK_BOOST_VDDN 0xB0
#define EPD_TFT_VCOM_POWER 0xB1
#define EPD_EN_BUF 0xB6
#define EPD_BOOST_VDDP_EN 0xB7
#define EPD_CCSET 0xE0
#define EPD_PWS 0xE3
#define EPD_CMD66 0xF0

#define EPD_FIRST_DATA_PACKET 1
#define EPD_NOT_FIRST_DATA_PACKET 0

#define EPD_PTLW_ENABLE 0x01
#define EPD_PTLW_DISABLE 0x00

// Image buffer for sending image
#define EPD_IMAGE_DATA_BUFFER_SIZE 8192 // MCU RAM Size (800*720/2) reserve for one driver IC

typedef struct {
    comms_t *comms;
    unsigned char *buffer; // Pointer to a buffer of size EPD_IMAGE_DATA_BUFFER_SIZE
    char partial_window_update_status;
} gdep133c02_t;

void gdep133c02_init(gdep133c02_t *self, comms_t *comms, unsigned char *buffer);
void gdep133c02_hardware_reset(gdep133c02_t *self);
void gdep133c02_set_pin_cs_all(gdep133c02_t *self, unsigned int set_level);
void gdep133c02_set_pin_cs(gdep133c02_t *self, unsigned char cs_number, unsigned int set_level);
void gdep133c02_check_busy_high(gdep133c02_t *self);
void gdep133c02_check_busy_low(gdep133c02_t *self);
void gdep133c02_init_epd(gdep133c02_t *self);
void gdep133c02_write_epd(gdep133c02_t *self, unsigned char epd_command, unsigned char *epd_data, unsigned int epd_data_length);
void gdep133c02_read_epd(gdep133c02_t *self, unsigned char epd_command, unsigned char *epd_data, unsigned int epd_data_length);
void gdep133c02_write_epd_command(gdep133c02_t *self, unsigned char epd_command);
void gdep133c02_write_epd_data(gdep133c02_t *self, unsigned char *epd_data, unsigned int epd_data_length);
void gdep133c02_display(gdep133c02_t *self);
void gdep133c02_display_color(gdep133c02_t *self, unsigned char color_select);
void gdep133c02_write_epd_image(gdep133c02_t *self, unsigned char csx, unsigned char const *image_data, unsigned long image_data_length);
void gdep133c02_display_color_bar(gdep133c02_t *self);
unsigned char gdep133c02_check_driver_ic_status(gdep133c02_t *self);
char gdep133c02_partial_window_update_with_image_data(
    gdep133c02_t *self,
    unsigned char csx, unsigned char const *image_data, unsigned long data_size,
    unsigned int x_start, unsigned int y_start, unsigned int x_pixel,
    unsigned int y_line, unsigned char display_enable);
char gdep133c02_partial_window_update_without_image_data(
    gdep133c02_t *self,
    unsigned char csx, unsigned int x_start, unsigned int y_start,
    unsigned int x_pixel, unsigned int y_line, unsigned char epd_display_enable);

void gdep133c02_pic_display_test(gdep133c02_t *self, const unsigned char *num);
void gdep133c02_draw_checkerboard(gdep133c02_t *self);

#endif // #ifndef __GDEP133C02_H__
