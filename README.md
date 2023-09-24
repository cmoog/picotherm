# picotherm

> Work in progress.

This repository contains source code and PCB schematics for a minimal, HomeKit
thermostat built with $30 in components.

## Design goals

- Minimal physical controls
  - display for target temp and current temp, offload everything else to
    wireless control.
- <= $30 in components.
- No separate wireless relay for local network connectivity.
- Apple Home + Google Home compatible.

## TODO

- [ ] Support fan wire.
- [ ] Commit schematic and PCB layout files.
- [ ] Matter IP protocol compatibility.

## Bill of Materials

| part                                                                                                    | name                    | $ / unit | quantity |
| ------------------------------------------------------------------------------------------------------- | ----------------------- | -------- | -------- |
| [16608263](https://www.digikey.com/en/products/detail/raspberry-pi/SC0918/16608263)                     | Pi Pico W               | 6.00     | 1        |
| [6136306](https://www.digikey.com/en/products/detail/bosch-sensortec/BME280/6136306)                    | Thermometer             | 6.42     | 1        |
|                                                                                                         | Electromechanical relay | 6.18     | 2        |
| [408212](https://www.digikey.com/en/products/detail/liteon/LTD-4708JR/408212)                           | 2 digit display         | 2.09     | 2        |
| order from [pcbway.com](https://pcbway.com)                                                             | PCB                     | _        | 1        |

## Schematic

> TODO

## Building from source

```sh
nix build
cp ./result/bin/therm.ef2 .
```

Flash `therm.ef2` to the Pi Pico W over USB by following the instructions in the
Pico W documentation for placing the device into flashing mode.
