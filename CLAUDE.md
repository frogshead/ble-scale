# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run (requires .env file)
cargo run

# Check without building
cargo check

# Run tests
cargo test
```

## Configuration

The app reads configuration from a `.env` file (gitignored). Required variables:
- `ADDRESS` — Bluetooth MAC address of the MI Scale 2 (e.g. `C8:47:8C:D1:7F:DC`)
- `INFLUXDB_USERNAME` — InfluxDB username
- `INFLUXDB_PASSWORD` — InfluxDB password

## Architecture

Single-binary Rust application (`src/main.rs`) that:

1. Connects to a Xiaomi MI Scale 2 via Bluetooth LE using `btleplug`
2. Scans for BLE advertisements filtered by the Body Composition service UUID (`0x181d`)
3. Parses weight from `ServiceDataAdvertisement` events: bytes `[2]<<8 | [1]` divided by 200 gives kg
4. (Intended) writes weight measurements to InfluxDB v2 via `influx_db_client`

The main async loop listens to `CentralEvent` from the BLE adapter. Weight parsing happens in the `ServiceDataAdvertisement` branch. The `store_weight` function and InfluxDB write path are stubbed out but not yet wired in.

## Infrastructure

`docker-compose.yml` runs a local InfluxDB v2 instance + Telegraf. The InfluxDB setup uses:
- bucket: `mybucket`, org: `myorg`, token: `mytoken` (for local dev)
- Telegraf config in `telegraf.conf` writes to `http://influxdb:8086`

## Key BLE UUIDs (MI Scale 2)

| UUID | Characteristic |
|------|---------------|
| `0x2A9D` | Weight Measurement (INDICATE) |
| `0x2A2B` | Current Time (READ/WRITE) |
| `0x181D` | Body Composition Service (used as scan filter) |
| `00002A2F-...` | Weight History (WRITE/NOTIFY) |

The weight history command is `[0x01, 0xFF, 0xFF, 0xFF, 0xFF]` sent to the weight history characteristic. Reference: [openScale wiki](https://github.com/oliexdev/openScale/wiki/Xiaomi-Bluetooth-Mi-Scale).
