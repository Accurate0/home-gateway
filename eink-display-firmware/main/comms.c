#include "comms.h"
#include "pindefine.h"
#include <driver/gpio.h>
#include <stdio.h>
#include <string.h>

#define SPI SPI3_HOST
#define SPI_MAX_BUFFER_SIZE 32768

esp_err_t comms_init_spi(comms_t *self) {
  esp_err_t status;

  spi_bus_config_t spi_bus_config = {.data0_io_num = SPI_Data0,
                                     .data1_io_num = SPI_Data1,
                                     .sclk_io_num = SPI_CLK,
                                     .quadwp_io_num = -1,
                                     .quadhd_io_num = -1,
                                     .flags = SPICOMMON_BUSFLAG_MASTER,
                                     .max_transfer_sz = SPI_MAX_BUFFER_SIZE};

  spi_device_interface_config_t spi_device_config = {
      .command_bits = 8,
      .clock_speed_hz = SPI_MASTER_FREQ_10M, // Clock out at 10 MHz
      .duty_cycle_pos = 128,                 // 50% duty cycle
      .queue_size = 7, // We want to be able to queue 7 transactions at a time
      .cs_ena_posttrans =
          3, // Keep the CS low 3 cycles after transaction, to stop slave from
             // missing the last bit when CS has less propagation delay than CLK
  };

  status = spi_bus_initialize(SPI, &spi_bus_config, SPI_DMA_CH_AUTO);
  ESP_ERROR_CHECK(status);
  // Attach the LCD to the SPI bus
  status = spi_bus_add_device(SPI, &spi_device_config, &self->spi);
  ESP_ERROR_CHECK(status);

#if COMMS_SHOW_LOG
  if (status == ESP_OK) {
    printf("comms_init_spi() has been executed. \r\n");
  }
#endif

  return status;
}

void comms_init_gpio(comms_t *self) {
  esp_err_t status;

  gpio_config_t io_config = {};

  io_config.pin_bit_mask = ((1ULL << EPD_RST) | (1ULL << SPI_CS0) |
                            (1ULL << SPI_CS1) | (1ULL << LOAD_SW));
  io_config.mode = GPIO_MODE_OUTPUT;
  io_config.pull_up_en = GPIO_PULLUP_ENABLE;
  status = gpio_config(&io_config);

#if COMMS_SHOW_LOG
  if (status == ESP_OK) {
    printf("gpio_config(&io_config) has been executed. \r\n");
  }
#endif

  io_config.pin_bit_mask = (1ULL << EPD_BUSY);
  io_config.intr_type = GPIO_INTR_NEGEDGE;
  io_config.mode = GPIO_MODE_INPUT;
  io_config.pull_down_en = GPIO_PULLDOWN_DISABLE;
  io_config.pull_up_en = GPIO_PULLUP_DISABLE;
  status = gpio_config(&io_config);

#if COMMS_SHOW_LOG
  if (status == ESP_OK) {
    printf("comms_init_gpio() has been executed. \r\n");
  }
#endif
}

void comms_delay_ms(comms_t *self, unsigned int delay_time) {
  vTaskDelay(delay_time / portTICK_PERIOD_MS);
}

esp_err_t comms_spi_transmit_command(comms_t *self, unsigned char command_buf) {
  esp_err_t status;
  spi_transaction_t trans;

  memset(&trans, 0, sizeof(trans));
  trans.cmd = command_buf;

  trans.length = 0;
  trans.tx_buffer = NULL;
  status = spi_device_transmit(self->spi, &trans);
  assert(status == ESP_OK);

  return status;
}

esp_err_t comms_spi_transmit_data(comms_t *self, unsigned char *data_buffer,
                                  unsigned long data_length) {
  esp_err_t status = 0;

  spi_transaction_ext_t trans_ext;

  while (data_length >= SPI_MAX_BUFFER_SIZE) {
    memset(&trans_ext, 0, sizeof(trans_ext));
    trans_ext.command_bits = 0;
    trans_ext.base.length = SPI_MAX_BUFFER_SIZE * 8;
    // The trans_ext.base.length unit is bit, so the SPI_MAX_BUFFER_SIZE must be
    // multiplied by 8.
    trans_ext.base.tx_buffer = data_buffer;
    trans_ext.base.flags = SPI_TRANS_VARIABLE_CMD;
    status = spi_device_transmit(self->spi, &trans_ext.base);
    data_length -= SPI_MAX_BUFFER_SIZE;
    data_buffer += SPI_MAX_BUFFER_SIZE;
  }

  if (data_length > 0) {
    memset(&trans_ext, 0, sizeof(trans_ext));
    trans_ext.command_bits = 0;
    trans_ext.base.length = data_length * 8;
    // The trans_ext.base.length unit is bit, so the data_length must be
    // multiplied by 8.
    trans_ext.base.tx_buffer = data_buffer;
    trans_ext.base.flags = SPI_TRANS_VARIABLE_CMD;
    status = spi_device_transmit(self->spi, &trans_ext.base);
  }

  return status;
}

esp_err_t comms_spi_receive_data(comms_t *self, unsigned char *data_buffer,
                                 unsigned long data_length) {
  esp_err_t status = 0;

  spi_transaction_ext_t trans_ext;

  while (data_length > SPI_MAX_BUFFER_SIZE) {
    memset(&trans_ext, 0, sizeof(trans_ext));

    trans_ext.command_bits = 0;
    trans_ext.base.length = SPI_MAX_BUFFER_SIZE * 8;
    // The trans_ext.base.length unit is bit, so the SPI_MAX_BUFFER_SIZE must be
    // multiplied by 8.
    trans_ext.base.rx_buffer = data_buffer;
    trans_ext.base.rxlength = data_length * 8;
    trans_ext.base.flags = SPI_TRANS_VARIABLE_CMD;
    status = spi_device_transmit(self->spi, &trans_ext.base);
    data_length -= SPI_MAX_BUFFER_SIZE;
    data_buffer += SPI_MAX_BUFFER_SIZE;
  }

  if (data_length > 0) {
    memset(&trans_ext, 0, sizeof(trans_ext));

    trans_ext.command_bits = 0;
    trans_ext.base.length = data_length * 8;
    // The trans_ext.base.length unit is bit, so the data_length must be
    // multiplied by 8.
    trans_ext.base.rx_buffer = data_buffer;
    trans_ext.base.rxlength = data_length * 8;
    trans_ext.base.flags = SPI_TRANS_VARIABLE_CMD;
    status = spi_device_transmit(self->spi, &trans_ext.base);
  }

  return status;
}

esp_err_t comms_spi_transmit_large_data(comms_t *self,
                                        unsigned char command_buf,
                                        unsigned char *data_buffer,
                                        unsigned long data_length) {
  esp_err_t status = 0;
  spi_transaction_t trans;
  spi_transaction_ext_t trans_ext;

  unsigned char first_packet = 1;

  while (data_length > SPI_MAX_BUFFER_SIZE) {
    if (first_packet) {
      memset(&trans, 0, sizeof(trans));
      trans.cmd = command_buf;
      trans.length = SPI_MAX_BUFFER_SIZE * 8;
      // The trans.length unit is bit, so the SPI_MAX_BUFFER_SIZE must be
      // multiplied by 8.
      trans.tx_buffer = data_buffer;
      trans.rx_buffer = NULL;
      status = spi_device_transmit(self->spi, &trans);
      first_packet = 0;
    } else {
      memset(&trans_ext, 0, sizeof(trans_ext));
      trans_ext.command_bits = 0;
      trans_ext.base.length = SPI_MAX_BUFFER_SIZE * 8;
      // The trans_ext.base.length unit is bit, so the data_length must be
      // multiplied by 8.
      trans_ext.base.tx_buffer = data_buffer;
      trans_ext.base.flags = SPI_TRANS_VARIABLE_CMD;
      status = spi_device_transmit(self->spi, &trans_ext.base);
    }

    data_length -= SPI_MAX_BUFFER_SIZE;
    data_buffer += SPI_MAX_BUFFER_SIZE;
  }

  if (data_length > 0) {
    if (first_packet) {
      memset(&trans, 0, sizeof(trans));
      trans.cmd = command_buf;
      trans.length = data_length * 8;
      // The trans.length unit is bit, so the data_length must be multiplied
      // by 8.
      trans.tx_buffer = data_buffer;
      trans.rx_buffer = NULL;
      status = spi_device_transmit(self->spi, &trans);
      first_packet = 0;
    } else {
      memset(&trans_ext, 0, sizeof(trans_ext));
      trans_ext.command_bits = 0;
      trans_ext.base.length = data_length * 8;
      // The trans_ext.base.length unit is bit, so the data_length must be
      // multiplied by 8.
      trans_ext.base.tx_buffer = data_buffer;
      trans_ext.base.flags = SPI_TRANS_VARIABLE_CMD;
      status = spi_device_transmit(self->spi, &trans_ext.base);
    }
  }

  return status;
}

esp_err_t comms_spi_transmit(comms_t *self, unsigned char command_buf,
                             unsigned char *data_buffer,
                             unsigned int data_length) {
  esp_err_t status = 0;
  spi_transaction_t trans;

  memset(&trans, 0, sizeof(trans));

  if (data_length < SPI_MAX_BUFFER_SIZE) {
    trans.cmd = command_buf;
    trans.length = data_length * 8;
    // The trans.length unit is bit, so the data_length must be multiplied by 8.
    trans.tx_buffer = data_buffer;
    trans.rx_buffer = NULL;
    status = spi_device_transmit(self->spi, &trans);

  } else
    status = -1; // The data_length is over the SPI_MAX_BUFFER_SIZE

  return status;
}

esp_err_t comms_spi_receive(comms_t *self, unsigned char command_buf,
                            unsigned char *data_buffer,
                            unsigned int data_length) {
  esp_err_t status = 0;

  spi_transaction_t trans;

  memset(&trans, 0, sizeof(trans));

  if (data_length < SPI_MAX_BUFFER_SIZE) {
    trans.cmd = command_buf;
    trans.length = data_length * 8;
    // The trans.length unit is bit, so the data_length must be multiplied by 8.
    trans.rxlength = data_length * 8;
    // The trans.rxlength unit is bit, so the data_length must be multiplied
    // by 8.
    trans.rx_buffer = data_buffer;

    //==== SPI Transmit Command & Receive Data ====
    status = spi_device_transmit(self->spi, &trans);
    assert(status == ESP_OK);
  } else
    status = -1; // The data_length is over the SPI_MAX_BUFFER_SIZE

  return status;
}

void comms_set_gpio_level(comms_t *self, unsigned char pin_number,
                          unsigned char voltage_level) {
  //==== Set GPIO voltage level ====
  gpio_set_level(pin_number, voltage_level);
}

unsigned char comms_get_gpio_level(comms_t *self, unsigned char pin_number) {
  unsigned char voltage_level;
  //==== Get GPIO voltage level ====
  voltage_level = gpio_get_level(pin_number);

  return voltage_level;
}
