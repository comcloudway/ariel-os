# wifi-dpp

## About

Example ArielOS project showing how to enable DPP support using `wifi-dpp-rs`

## How to run

In this directory, run

```sh
laze build -b eespressif-esp32-s3-devkitc-1 run
```

You can provide an existing SecretKey by setting `CONFIG_WIFI_DPP_SECRET_KEY_FILE="<./path/to/file>"`
