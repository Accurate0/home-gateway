idf_component_register(SRCS "main.c" "comms.c" "GDEP133C02.c"
    PRIV_REQUIRES esp_wifi esp_driver_gpio esp_driver_spi nvs_flash
    INCLUDE_DIRS ".")
