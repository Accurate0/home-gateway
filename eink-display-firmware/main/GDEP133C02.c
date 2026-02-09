#include "GDEP133C02.h"
#include "comms.h"
#include "pindefine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const unsigned char spi_cs_pin[2] = {SPI_CS0, SPI_CS1};
const unsigned char psr_v[2] = {0xDF, 0x69};
const unsigned char pwr_v[6] = {0x0F, 0x00, 0x28, 0x2C, 0x28, 0x38};
const unsigned char pof_v[1] = {0x00};
const unsigned char drf_v[1] = {0x01};
const unsigned char cdi_v[1] = {0xF7};
const unsigned char tcon_v[2] = {0x03, 0x03};
const unsigned char tres_v[4] = {0x04, 0xB0, 0x03, 0x20};
const unsigned char cmd66_v[6] = {0x49, 0x55, 0x13, 0x5D, 0x05, 0x10};
const unsigned char en_buf_v[1] = {0x07};
const unsigned char ccset_v[1] = {0x01};
const unsigned char pws_v[1] = {0x22};
const unsigned char an_tm_v[9] = {0xC0, 0x1C, 0x1C, 0xCC, 0xCC,
                                  0xCC, 0x15, 0x15, 0x55};

const unsigned char agid_v[1] = {0x10};

const unsigned char btst_p_v[2] = {0xE8, 0x28};
const unsigned char boost_vddp_en_v[1] = {0x01};
const unsigned char btst_n_v[2] = {0xE8, 0x28};
const unsigned char buck_boost_vddn_v[1] = {0x01};
const unsigned char tft_vcom_power_v[1] = {0x02};

void gdep133c02_init(gdep133c02_t *self, comms_t *comms,
                     unsigned char *buffer) {
  self->comms = comms;
  self->buffer = buffer;
  self->partial_window_update_status = COMMS_DONE;
}

//================== GPIO Setting ====================================
static void reset_pin(gdep133c02_t *self, unsigned int pin_status) {
  comms_set_gpio_level(self->comms, EPD_RST, pin_status);
}

void gdep133c02_set_pin_cs_all(gdep133c02_t *self, unsigned int set_level) {
  unsigned char i;
  for (i = 0; i < 2; i++) {
    comms_set_gpio_level(self->comms, spi_cs_pin[i], set_level);
  }
}

void gdep133c02_set_pin_cs(gdep133c02_t *self, unsigned char cs_number,
                           unsigned int set_level) {
  comms_set_gpio_level(self->comms, spi_cs_pin[cs_number], set_level);
}

void gdep133c02_check_busy_high(gdep133c02_t *self) // If BUSYN=0 then waiting
{
  while (!(comms_get_gpio_level(self->comms, EPD_BUSY)))
    ;
}

void gdep133c02_check_busy_low(gdep133c02_t *self) // If BUSYN=1 then waiting
{
  while (comms_get_gpio_level(self->comms, EPD_BUSY))
    ;
}
//====================================================================

void gdep133c02_hardware_reset(gdep133c02_t *self) {
  reset_pin(self, GPIO_LOW);
  comms_delay_ms(self->comms, 20);
  reset_pin(self, GPIO_HIGH);
  comms_delay_ms(self->comms, 20);
}

void gdep133c02_write_epd(gdep133c02_t *self, unsigned char epd_command,
                          unsigned char *epd_data,
                          unsigned int epd_data_length) {
  comms_spi_transmit(self->comms, epd_command, epd_data, epd_data_length);
}

void gdep133c02_read_epd(gdep133c02_t *self, unsigned char epd_command,
                         unsigned char *epd_data,
                         unsigned int epd_data_length) {
  comms_spi_receive(self->comms, epd_command, epd_data, epd_data_length);
}

void gdep133c02_write_epd_command(gdep133c02_t *self,
                                  unsigned char epd_command) {
  comms_spi_transmit_command(self->comms, epd_command);
}

void gdep133c02_write_epd_data(gdep133c02_t *self, unsigned char *epd_data,
                               unsigned int epd_data_length) {
  comms_spi_transmit_data(self->comms, epd_data, epd_data_length);
}

void gdep133c02_init_epd(gdep133c02_t *self) {
  gdep133c02_hardware_reset(self);
  gdep133c02_check_busy_high(self);
  // checkBusyLow();

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_AN_TM, (unsigned char *)an_tm_v,
                       sizeof(an_tm_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_CMD66, (unsigned char *)cmd66_v,
                       sizeof(cmd66_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_PSR, (unsigned char *)psr_v, sizeof(psr_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_CDI, (unsigned char *)cdi_v, sizeof(cdi_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_TCON, (unsigned char *)tcon_v, sizeof(tcon_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_AGID, (unsigned char *)agid_v, sizeof(agid_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_PWS, (unsigned char *)pws_v, sizeof(pws_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_CCSET, (unsigned char *)ccset_v,
                       sizeof(ccset_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_TRES, (unsigned char *)tres_v, sizeof(tres_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_PWR, (unsigned char *)pwr_v, sizeof(pwr_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_EN_BUF, (unsigned char *)en_buf_v,
                       sizeof(en_buf_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_BTST_P, (unsigned char *)btst_p_v,
                       sizeof(btst_p_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_BOOST_VDDP_EN,
                       (unsigned char *)boost_vddp_en_v,
                       sizeof(boost_vddp_en_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_BTST_N, (unsigned char *)btst_n_v,
                       sizeof(btst_n_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_BUCK_BOOST_VDDN,
                       (unsigned char *)buck_boost_vddn_v,
                       sizeof(buck_boost_vddn_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_set_pin_cs(self, 0, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_TFT_VCOM_POWER,
                       (unsigned char *)tft_vcom_power_v,
                       sizeof(tft_vcom_power_v));
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

#if COMMS_SHOW_LOG
  printf("initEPD() has been executed. \r\n");
#endif
}

unsigned char gdep133c02_check_driver_ic_status(gdep133c02_t *self) {
  unsigned char csx, status = COMMS_DONE;
  unsigned char data_buf[3];

  for (csx = 0; csx < 2; csx++) {
    memset(data_buf, 0, sizeof(data_buf));
    gdep133c02_set_pin_cs(self, csx, GPIO_LOW);
    gdep133c02_read_epd(self, 0xF2, data_buf, sizeof(data_buf));
    gdep133c02_set_pin_cs(self, csx, GPIO_HIGH);
#if COMMS_SHOW_LOG
    printf("Driver IC [%d] = 0x%02X 0x%02X 0x%02X \r\n", csx, data_buf[0],
           data_buf[1], data_buf[2]);
#endif
    if ((data_buf[0] & 0x01) == 0x01) {
#if COMMS_SHOW_LOG
      printf("Driver IC [%d] is ready. \r\n", csx);
#endif
    } else {
#if COMMS_SHOW_LOG
      printf("Driver IC [%d] did not reply. \r\n", csx);
#endif
      status = COMMS_ERROR;
    }
  }

  return status;
}

void gdep133c02_display(gdep133c02_t *self) {

#if COMMS_SHOW_LOG
  printf("Write PON \r\n");
#endif
  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd_command(self, EPD_PON);
  gdep133c02_check_busy_high(self);
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

#if COMMS_SHOW_LOG
  printf("Write DRF \r\n");
#endif
  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  comms_delay_ms(self->comms, 30);
  gdep133c02_write_epd(self, EPD_DRF, (unsigned char *)drf_v, sizeof(drf_v));
  gdep133c02_check_busy_high(self);
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

#if COMMS_SHOW_LOG
  printf("Write POF \r\n");
#endif
  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd(self, EPD_POF, (unsigned char *)pof_v, sizeof(pof_v));
  gdep133c02_check_busy_high(self);
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);
#if COMMS_SHOW_LOG
  printf("Display Done!! \r\n");
#endif
}

void gdep133c02_display_color(gdep133c02_t *self, unsigned char color_select) {

  unsigned long i;

  memset(self->buffer, color_select, EPD_IMAGE_DATA_BUFFER_SIZE);

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd_command(self, EPD_DTM);
  for (i = 0; i < 480000 / EPD_IMAGE_DATA_BUFFER_SIZE; i++) {
    gdep133c02_write_epd_data(self, self->buffer, EPD_IMAGE_DATA_BUFFER_SIZE);
  }
  gdep133c02_write_epd_data(self, self->buffer,
                            480000 % EPD_IMAGE_DATA_BUFFER_SIZE);
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_display(self);

#if COMMS_SHOW_LOG
  printf("Display color complete. \r\n");
#endif
}

// Display screen parameters (already provided)
#define EPD_WIDTH 1200          // Total display width (pixels)
#define EPD_HEIGHT 1600         // Display height (pixels)
#define FIRST_PACK_SIZE 480000  // First data packet size (bytes)
#define TOTAL_IMAGE_SIZE 960000 // Total image data size (bytes)

void gdep133c02_pic_display_test(gdep133c02_t *self, const unsigned char *num) {
  unsigned int width, width1, height;
  // Calculate width and height using the same method as the second code
  width = (EPD_WIDTH % 2 == 0)
              ? (EPD_WIDTH / 2)
              : (EPD_WIDTH / 2 + 1); // Width per section (pixels)
  width1 = (width % 2 == 0)
               ? (width / 2)
               : (width / 2 +
                  1);  // Width per section (bytes, assuming 8 bits per pixel)
  height = EPD_HEIGHT; // Height (pixels)

  // Transfer data to the first section (main display)
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH); // Deselect all
  gdep133c02_set_pin_cs(self, 0, 0); // Select the first section (main display)
  gdep133c02_write_epd_command(self,
                               EPD_DTM); // Send data transfer mode command
  for (unsigned int i = 0; i < height; i++) {
    gdep133c02_write_epd_data(self, (unsigned char *)(num + i * width),
                              width1); // Send the first half of each row's data
    vTaskDelay(pdMS_TO_TICKS(1));      // Delay 1ms to avoid hardware overload
  }
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH); // Deselect

  // Transfer data to the second section (secondary display)
  gdep133c02_set_pin_cs(self, 1,
                        0); // Select the second section (secondary display)
  gdep133c02_write_epd_command(self,
                               EPD_DTM); // Send data transfer mode command
  for (unsigned int i = 0; i < height; i++) {
    gdep133c02_write_epd_data(
        self, (unsigned char *)(num + i * width + width1),
        width1);                  // Send the second half of each row's data
    vTaskDelay(pdMS_TO_TICKS(1)); // Delay 1ms
  }
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH); // Deselect

  // Refresh the display
  gdep133c02_display(self);          // Trigger display
  vTaskDelay(pdMS_TO_TICKS(10));     // Delay 10ms to ensure refresh completion
  printf("Rendering completed\r\n"); // Print completion message
}

void gdep133c02_draw_checkerboard(gdep133c02_t *self) {
  // Calculate the display's width and height
  unsigned int width = (EPD_WIDTH % 2 == 0)
                           ? (EPD_WIDTH / 2)
                           : (EPD_WIDTH / 2 + 1); // Width per section (pixels)
  unsigned int width1 = (width % 2 == 0)
                            ? (width / 2)
                            : (width / 2 + 1); // Width per section (bytes)
  unsigned int height = EPD_HEIGHT;            // Height (pixels)

  // Verify the calculation results
  if (width != 600 || width1 != 300 || height != 1600) {
    printf("Calculation error: Width=%u, Width1=%u, Height=%u\r\n", width,
           width1, height);
    return;
  }

  // Allocate buffer (each byte stores two pixels)
  unsigned int buffer_size = TOTAL_IMAGE_SIZE; // Total bytes
  unsigned char *num =
      (unsigned char *)malloc(buffer_size * sizeof(unsigned char));
  if (num == NULL) {
    printf("Memory allocation failed!\r\n");
    return;
  }

  // Initialize the buffer
  memset(num, 0, buffer_size);

  // Define checkerboard colors (6 colors)
  const unsigned char colors[6] = {
      EPD_BLACK,  // Black
      EPD_WHITE,  // White
      EPD_YELLOW, // Yellow
      EPD_RED,    // Red
      EPD_BLUE,   // Blue
      EPD_GREEN   // Green
  };

  // Calculate the size of each checkerboard cell
  const int grid_cols = 6; // 6 columns
  const int grid_rows = 8; // 8 rows
  const int cell_width =
      EPD_WIDTH / grid_cols; // Width of each cell: 1200 / 6 = 200 pixels
  const int cell_height =
      EPD_HEIGHT / grid_rows; // Height of each cell: 1600 / 8 = 200 pixels

  // Fill pixel data (horizontal scan, from right to left, top to bottom)
  for (unsigned int y = 0; y < height; y++) {
    for (unsigned int x = 0; x < EPD_WIDTH;
         x += 2) { // Process two pixels at a time
      // Calculate the cell position of the current pixel
      int grid_x = x / cell_width;  // Column index
      int grid_y = y / cell_height; // Row index

      // Calculate color index (cycle through colors based on cell position for
      // a staggered pattern)
      int color_index = (grid_x + grid_y) % 6;    // Staggered distribution
      unsigned char color1 = colors[color_index]; // Color of the first pixel

      // Second pixel (if x+1 is in the same cell, the color is the same)
      int grid_x2 = (x + 1) / cell_width;
      int color_index2 = (grid_x2 + grid_y) % 6;   // Staggered distribution
      unsigned char color2 = colors[color_index2]; // Color of the second pixel

      // Calculate buffer index (horizontal scan, from right to left)
      int new_x = (EPD_WIDTH - 2 - x); // From right to left
      int new_index = (y * width) + (new_x / 2);

      // Combine two pixels into one byte
      num[new_index] = (color1 << 4) | color2;
    }
  }

  // Call pic_display_test to display
  gdep133c02_pic_display_test(self, num);

  // Free the buffer
  free(num);
}

void gdep133c02_display_color_bar(gdep133c02_t *self) {
  unsigned long i;

  gdep133c02_set_pin_cs_all(self, GPIO_LOW);
  gdep133c02_write_epd_command(self, EPD_DTM);

  // BLACK
  memset(self->buffer, EPD_BLACK, EPD_IMAGE_DATA_BUFFER_SIZE);
  for (i = 0; i < 80000 / EPD_IMAGE_DATA_BUFFER_SIZE; i++) {
    gdep133c02_write_epd_data(self, self->buffer, EPD_IMAGE_DATA_BUFFER_SIZE);
  }
  gdep133c02_write_epd_data(self, self->buffer,
                            80000 % EPD_IMAGE_DATA_BUFFER_SIZE);
  // WHITE
  memset(self->buffer, EPD_WHITE, EPD_IMAGE_DATA_BUFFER_SIZE);
  for (i = 0; i < 80000 / EPD_IMAGE_DATA_BUFFER_SIZE; i++) {
    gdep133c02_write_epd_data(self, self->buffer, EPD_IMAGE_DATA_BUFFER_SIZE);
  }
  gdep133c02_write_epd_data(self, self->buffer,
                            80000 % EPD_IMAGE_DATA_BUFFER_SIZE);
  // YELLOW
  memset(self->buffer, EPD_YELLOW, EPD_IMAGE_DATA_BUFFER_SIZE);
  for (i = 0; i < 80000 / EPD_IMAGE_DATA_BUFFER_SIZE; i++) {
    gdep133c02_write_epd_data(self, self->buffer, EPD_IMAGE_DATA_BUFFER_SIZE);
  }
  gdep133c02_write_epd_data(self, self->buffer,
                            80000 % EPD_IMAGE_DATA_BUFFER_SIZE);
  // RED
  memset(self->buffer, EPD_RED, EPD_IMAGE_DATA_BUFFER_SIZE);
  for (i = 0; i < 80000 / EPD_IMAGE_DATA_BUFFER_SIZE; i++) {
    gdep133c02_write_epd_data(self, self->buffer, EPD_IMAGE_DATA_BUFFER_SIZE);
  }
  gdep133c02_write_epd_data(self, self->buffer,
                            80000 % EPD_IMAGE_DATA_BUFFER_SIZE);
  // BLUE
  memset(self->buffer, EPD_BLUE, EPD_IMAGE_DATA_BUFFER_SIZE);
  for (i = 0; i < 80000 / EPD_IMAGE_DATA_BUFFER_SIZE; i++) {
    gdep133c02_write_epd_data(self, self->buffer, EPD_IMAGE_DATA_BUFFER_SIZE);
  }
  gdep133c02_write_epd_data(self, self->buffer,
                            80000 % EPD_IMAGE_DATA_BUFFER_SIZE);
  // GREEN
  memset(self->buffer, EPD_GREEN, EPD_IMAGE_DATA_BUFFER_SIZE);
  for (i = 0; i < 80000 / EPD_IMAGE_DATA_BUFFER_SIZE; i++) {
    gdep133c02_write_epd_data(self, self->buffer, EPD_IMAGE_DATA_BUFFER_SIZE);
  }
  gdep133c02_write_epd_data(self, self->buffer,
                            80000 % EPD_IMAGE_DATA_BUFFER_SIZE);
  gdep133c02_set_pin_cs_all(self, GPIO_HIGH);

  gdep133c02_display(self);

#if COMMS_SHOW_LOG
  printf("Display color bar complete. \r\n");
#endif
}

void gdep133c02_write_epd_image(gdep133c02_t *self, unsigned char csx,
                                unsigned char const *image_data,
                                unsigned long image_data_length) {

  gdep133c02_set_pin_cs(self, csx, GPIO_LOW);
  comms_spi_transmit_large_data(self->comms, EPD_DTM,
                                (unsigned char *)image_data, image_data_length);
  gdep133c02_set_pin_cs(self, csx, GPIO_HIGH);

#if COMMS_SHOW_LOG
  printf("Writing data is completed. \r\n");
#endif
}

char gdep133c02_partial_window_update_with_image_data(
    gdep133c02_t *self, unsigned char csx, unsigned char const *image_data,
    unsigned long image_data_length, unsigned int x_start, unsigned int y_start,
    unsigned int x_pixel, unsigned int y_line,
    unsigned char epd_display_enable) {
  unsigned char status = COMMS_DONE;
  unsigned int hrst, hred, vrst, vred;
  unsigned char partial_window_data[9];

  hrst = x_start * 2;
  hred = (x_start + x_pixel) * 2 - 1; // The range is 0 ~ 1199
  vrst = y_start / 2;
  vred = (y_start + y_line) / 2 - 1; // The range is 0 ~ 799

#if COMMS_SHOW_LOG
  printf("csx = %d ; hrst = %d ; hred = %d ; vrst = %d ; vred = %d \r\n", csx,
         hrst, hred, vrst, vred);
#endif

  // hrst[10:0] = 8n (n = 0,1,2…)
  if (hrst % 8 != 0) {
    status = -1;
#if COMMS_SHOW_LOG
    printf("status = -1 ; There is a problem with x_start. \r\n");
#endif
  }
  // hred[10:0] = 8m+3 (m = 4,5,6…)
  else if ((hred - 7) % 8 != 0) {
    status = -2;
#if COMMS_SHOW_LOG
    printf("status = -2 ; There is a problem with x_pixel. \r\n");
#endif
  }
  //  x_start <= 584 ; x_pixel <= 600
  else if ((x_start > 584) | (x_pixel > 600)) {
    status = -3;
#if COMMS_SHOW_LOG
    printf("status = -3 ; x_start or x_pixel is over range. \r\n");
#endif
  }
  // hred - hrst + 1 >= 32 & hred + 1 <= 1200
  else if ((hred - hrst + 1 < 32) | (hred + 1 > 1200)) {
    status = -4;
#if COMMS_SHOW_LOG
    printf("status = -4 ; There is a problem with x_start & x_pixel. \r\n");
#endif
  } else if ((y_start + y_line) % 2 != 0) {
    status = -5;
#if COMMS_SHOW_LOG
    printf("status = -5 ; y_start + y_line must be an even number. \r\n");
#endif
  }
  // y_start <= 1596 ; y_line <= 1600
  else if ((y_start > 1596) | (y_line > 1600)) {
    status = -6;
#if COMMS_SHOW_LOG
    printf("status = -6 ; y_start or y_line is over range. \r\n");
#endif
  }
  // vrst - vred + 1 > 0 & vred + 1 <= 800
  else if (((int)(vred - vrst) + 1 <= 0) | (vred + 1 > 800)) {
    status = -7;
#if COMMS_SHOW_LOG
    printf("status = -7 ; There is a problem with y_start & y_line. \r\n");
#endif
  } else if (csx > 1) {
    status = -8;
#if COMMS_SHOW_LOG
    printf("status = -8 ; There is a problem with cxs. \r\n");
#endif
  } else {
    memset(partial_window_data, 0, sizeof(partial_window_data));
    partial_window_data[0] = (unsigned char)(hrst >> 8);
    partial_window_data[1] = (unsigned char)(hrst);
    partial_window_data[2] = (unsigned char)(hred >> 8);
    partial_window_data[3] = (unsigned char)(hred);
    partial_window_data[4] = (unsigned char)(vrst >> 8);
    partial_window_data[5] = (unsigned char)(vrst);
    partial_window_data[6] = (unsigned char)(vred >> 8);
    partial_window_data[7] = (unsigned char)(vred);
    partial_window_data[8] = EPD_PTLW_ENABLE;

    gdep133c02_set_pin_cs(self, csx, GPIO_LOW);
    gdep133c02_write_epd(self, EPD_CMD66, (unsigned char *)cmd66_v,
                         sizeof(cmd66_v));
    gdep133c02_set_pin_cs(self, csx, GPIO_HIGH);

    gdep133c02_set_pin_cs(self, csx, GPIO_LOW);
    gdep133c02_write_epd(self, EPD_PTLW, partial_window_data,
                         sizeof(partial_window_data));
    gdep133c02_set_pin_cs(self, csx, GPIO_HIGH);

    gdep133c02_set_pin_cs(self, csx, GPIO_LOW);
    comms_spi_transmit_large_data(
        self->comms, EPD_DTM, (unsigned char *)image_data, image_data_length);
    gdep133c02_set_pin_cs(self, csx, GPIO_HIGH);
  }

  if (status != COMMS_DONE) {
    self->partial_window_update_status = COMMS_ERROR;
#if COMMS_SHOW_LOG
    printf("partial_window_update_status = ERROR \r\n");
#endif
  }

  if (epd_display_enable) {
    if (self->partial_window_update_status == COMMS_DONE)
      gdep133c02_display(self);

    comms_delay_ms(self->comms, 300);

    //========================= Turn off PTLW =========================
    memset(partial_window_data, 0, sizeof(partial_window_data));
    partial_window_data[8] = EPD_PTLW_DISABLE;
    self->partial_window_update_status = COMMS_DONE;

    gdep133c02_set_pin_cs_all(self, GPIO_LOW);
    gdep133c02_write_epd(self, EPD_PTLW, partial_window_data,
                         sizeof(partial_window_data));
    gdep133c02_set_pin_cs_all(self, GPIO_HIGH);
    //=================================================================
  }

  return status;
}

char gdep133c02_partial_window_update_without_image_data(
    gdep133c02_t *self, unsigned char csx, unsigned int x_start,
    unsigned int y_start, unsigned int x_pixel, unsigned int y_line,
    unsigned char epd_display_enable) {
  unsigned char status = COMMS_DONE;
  unsigned int hrst, hred, vrst, vred;
  unsigned char partial_window_data[9];

  hrst = x_start * 2;
  hred = (x_start + x_pixel) * 2 - 1; // The range is 0 ~ 1199
  vrst = y_start / 2;
  vred = (y_start + y_line) / 2 - 1; // The range is 0 ~ 799

#if COMMS_SHOW_LOG
  printf("csx = %d ; hrst = %d ; hred = %d ; vrst = %d ; vred = %d \r\n", csx,
         hrst, hred, vrst, vred);
#endif

  // hrst[10:0] = 8n (n = 0,1,2…)
  if (hrst % 8 != 0) {
    status = -1;
#if COMMS_SHOW_LOG
    printf("status = -1 ; There is a problem with x_start. \r\n");
#endif
  }
  // hred[10:0] = 8m+3 (m = 4,5,6…)
  else if ((hred - 7) % 8 != 0) {
    status = -2;
#if COMMS_SHOW_LOG
    printf("status = -2 ; There is a problem with x_pixel. \r\n");
#endif
  }
  //  x_start <= 584 ; x_pixel <= 600
  else if ((x_start > 584) | (x_pixel > 600)) {
    status = -3;
#if COMMS_SHOW_LOG
    printf("status = -3 ; x_start or x_pixel is over range. \r\n");
#endif
  }
  // hred - hrst + 1 >= 32 & hred + 1 <= 1200
  else if ((hred - hrst + 1 < 32) | (hred + 1 > 1200)) {
    status = -4;
#if COMMS_SHOW_LOG
    printf("status = -4 ; There is a problem with x_start & x_pixel. \r\n");
#endif
  } else if ((y_start + y_line) % 2 != 0) {
    status = -5;
#if COMMS_SHOW_LOG
    printf("status = -5 ; y_start + y_line must be an even number. \r\n");
#endif
  }
  // y_start <= 1596 ; y_line <= 1600
  else if ((y_start > 1596) | (y_line > 1600)) {
    status = -6;
#if COMMS_SHOW_LOG
    printf("status = -6 ; y_start or y_line is over range. \r\n");
#endif
  }
  // vrst - vred + 1 > 0 & vred + 1 <= 800
  else if (((int)(vred - vrst) + 1 <= 0) | (vred + 1 > 800)) {
    status = -7;
#if COMMS_SHOW_LOG
    printf("status = -7 ; There is a problem with y_start & y_line. \r\n");
#endif
  } else if (csx > 1) {
    status = -8;
#if COMMS_SHOW_LOG
    printf("status = -8 ; There is a problem with cxs. \r\n");
#endif
  } else {
    memset(partial_window_data, 0, sizeof(partial_window_data));
    partial_window_data[0] = (unsigned char)(hrst >> 8);
    partial_window_data[1] = (unsigned char)(hrst);
    partial_window_data[2] = (unsigned char)(hred >> 8);
    partial_window_data[3] = (unsigned char)(hred);
    partial_window_data[4] = (unsigned char)(vrst >> 8);
    partial_window_data[5] = (unsigned char)(vrst);
    partial_window_data[6] = (unsigned char)(vred >> 8);
    partial_window_data[7] = (unsigned char)(vred);
    partial_window_data[8] = EPD_PTLW_ENABLE;

    gdep133c02_set_pin_cs(self, csx, GPIO_LOW);
    gdep133c02_write_epd(self, EPD_CMD66, (unsigned char *)cmd66_v,
                         sizeof(cmd66_v));
    gdep133c02_set_pin_cs(self, csx, GPIO_HIGH);

    gdep133c02_set_pin_cs(self, csx, GPIO_LOW);
    gdep133c02_write_epd(self, EPD_PTLW, partial_window_data,
                         sizeof(partial_window_data));
    gdep133c02_set_pin_cs(self, csx, GPIO_HIGH);
  }

  if (status != COMMS_DONE) {
    self->partial_window_update_status = COMMS_ERROR;
#if COMMS_SHOW_LOG
    printf("partial_window_update_status = ERROR \r\n");
#endif
  }

  if (epd_display_enable) {
    if (self->partial_window_update_status == COMMS_DONE)
      gdep133c02_display(self);

    comms_delay_ms(self->comms, 300);

    //========================= Turn off PTLW =========================
    memset(partial_window_data, 0, sizeof(partial_window_data));
    partial_window_data[8] = EPD_PTLW_DISABLE;
    self->partial_window_update_status = COMMS_DONE;

    gdep133c02_set_pin_cs_all(self, GPIO_LOW);
    gdep133c02_write_epd(self, EPD_PTLW, partial_window_data,
                         sizeof(partial_window_data));
    gdep133c02_set_pin_cs_all(self, GPIO_HIGH);
    //=================================================================
  }

  return status;
}
