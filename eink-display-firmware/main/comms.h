#ifndef __COMM_H__
#define __COMM_H__

#include "driver/spi_master.h"
#include "esp_err.h"
#include "freertos/task.h"

#define COMMS_ERROR 1
#define COMMS_DONE 0
#define COMMS_SHOW_LOG 1

typedef struct {
  spi_device_handle_t spi;
} comms_t;

esp_err_t comms_init_spi(comms_t *self);
void comms_init_gpio(comms_t *self);
void comms_delay_ms(comms_t *self, unsigned int delay_time);
esp_err_t comms_spi_transmit_command(comms_t *self, unsigned char command_buf);
esp_err_t comms_spi_transmit_data(comms_t *self, unsigned char *data_buffer,
                                  unsigned long data_length);
esp_err_t comms_spi_receive_data(comms_t *self, unsigned char *data_buffer,
                                 unsigned long data_length);
esp_err_t comms_spi_transmit_large_data(comms_t *self,
                                        unsigned char command_buf,
                                        unsigned char *data_buffer,
                                        unsigned long data_length);
esp_err_t comms_spi_transmit(comms_t *self, unsigned char command_buf,
                             unsigned char *data_buffer,
                             unsigned int data_length);
esp_err_t comms_spi_receive(comms_t *self, unsigned char command_buf,
                            unsigned char *data_buffer,
                            unsigned int data_length);
void comms_set_gpio_level(comms_t *self, unsigned char pin_number,
                          unsigned char voltage_level);
unsigned char comms_get_gpio_level(comms_t *self, unsigned char pin_number);

#endif // #ifndef __COMM_H__
