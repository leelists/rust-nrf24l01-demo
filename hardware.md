
# LilyGo v1.0 POE

### Pinout

|       |        | NRF24L01 |      |
|-------|--------|----------|------|
| GPIO39 | GPIO36 | NC       | NC   |
| GPIO35 | GPIO34 | NC       | NC   | 
| GPIO32 | GPIO16 | NC       | NC   |
| GPIO12 | GPIO33 | MISO     | IRQ  |
| GPIO15 | GPIO4  | SCK      | MOSI |
| GPIO14 | GPIO2  | CE       | CS  |
| GND   | 3V3    | GND      | 3V3  |
| GND   | 3V3    | E        | E    |


# Wiring

| NRF24l01 | LilyGo |
|----------|--------|
| GND      | GND    |
| 3V3      | 3V3    |
| CE       | GPIO14 |
| CS       | GPIO2  |
| SCK      | GPIO15 |
| MOSI     | GPIO4  |
| MISO     | GPIO12 |
| IRQ      | GPIO33 |

# NRF24l01

![](nrf24l01.png)

