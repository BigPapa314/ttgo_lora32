# sx127x_rs

A platform-agnostic driver for Semtech SX1276/77/78/79 based boards. It supports any device that implements the embedded-hal traits. Devices are connected over SPI and require an extra GPIO pin for RESET.

This project implements the communication to the sx127x controller using the embedded-hal v1.0.0 crate.

It is tested using the [ttgo lora32 (LILYGO T3_V1.6.1)](https://docs.nordicsemi.com/bundle/ncs-latest/page/zephyr/boards/lilygo/ttgo_lora32/doc/index.html) board. The test project can be found at the *ttgo_sx1276* project in the [ttgo_lora32](https://github.com/BigPapa314/ttgo_lora32) workspace.
