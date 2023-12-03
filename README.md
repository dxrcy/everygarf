# EveryGarf - Comic Downloader

Concurrently download every Garfield comic to date.

## Stats

- Download size: 4.8GB
- Download time: ~20m[*](#download-time)
- Images: >16,400

# Installation

## Binary

[Download latest release](https://github.com/darccyy/everygarf/releases/latest) (executable)

## Cargo (from source)

Install from source with `cargo`.
Eventually I will put this on [crates.io](https://crates.io)

```sh
cargo install --git https://github.com/darccyy/everygarf
```

Or clone repository, and install with `cargo`.

```sh
git clone https://github.com/darccyy/everygarf
cargo install --path everygarf
```

# Usage

```sh
# Help information
everygarf --help

# Download to default folder ('garfield' in user pictures directory)
everygarf

# Change some options
everygarf ~/Pictures/garfield --remove-all --notify-fail --attempts 20 --timeout 30 --jobs 40 --max 300
```

# About

## Download time

Download time was tested a few times (since v2.0), with varying values for `--jobs`, `--timeout`, and `--attempts`.
Speed is obviously very dependent on your ping and download speed.
Per some basic testing, increasing concurrency seems to have diminishing returns past `--jobs 25`.

![Graph of download speed to job count, showing exponential falloff from range 1-20, then settling on a similar rate from range 20-100](./image/download-speed-graph.png)

## API

Since an official Garfield comic API could not be found, this program scrapes [gocomics.com](https://www.gocomics.com/garfield/1978/6/19), and finds the [assets.amuniversal.com](https://assets.amuniversal.com/aead3a905f69012ee3c100163e41dd5b) link.
This requires 2 HTTP requests per comic.
The files hosted at [picayune.uclick.com](https://picayune.uclick.com/comics/ga/1978/ga780619.gif), while only requiring 1 request each, have been found to be very inconsistent and unstable, therefore are not used.

## Possible speed optimizations

As mentioned above, since each image requires 2 HTTP requests, the program's speed is almost entirely dependent on internet speed.
This program attempts to utilize as much concurrency as possible.
The only forseeable optimization to this program would be using a different web API.

## Proxy service

Default proxy url is `https://proxy.darcy-700.worker.dev`, a simple Cloudflare worker.

### Setup a custom proxy service with Cloudflare worker

Setup a Cloudflare worker with [https://github.com/Zibri/cloudflare-cors-anywhere](Zibri/cloudflare-cors-anywhere), and pass in the url with `--proxy`.

### Disable proxy

Use `--no-proxy` argument.
This is not recommended, you will most likely get immediately rate limited by [gocomics.com](https://www.gocomics.com/garfield/1978/6/19).

# Automatically Running with Systemd Timer

For systems with `systemd`.
Installs user service and timer to `~/.config/systemd/user`.

This assumes that `everygarf` is already [installed with `cargo`](#cargo-from-source).
Otherwise, binary path must be changed in `ExecStart` field in `everygarf.service`.

```sh
#!/bin/sh
# 1. Navigate to user systemd config
cd ~/.config/systemd/user || exit 1
# 2. Create service file
# ExecStart path must be absolute, $HOME is interpolated on file create
# Maximum 50 images at a time
echo "\
[Unit]
Description=Run EveryGarf program to download Garfield comics
[Service]
ExecStart=$HOME/.cargo/bin/everygarf --jobs 10 --max 50 --notify-fail
[Install]
WantedBy=everygarf.timer\
" > everygarf.service
# 3. Create timer file
# Runs shortly after each boot, and every 3 hours
echo "\
[Unit]
Description=Timer for EveryGarf service
[Timer]
OnBootSec=2min
OnUnitActiveSec=3h
Unit=everygarf.service
[Install]
WantedBy=timers.target\
" > everygarf.timer
# 4. Enable and start with systemd
systemctl --user daemon-reload
systemctl --user enable everygarf.timer
systemctl --user start everygarf.timer
```

> View logs of `everygarf.service` with `journalctl --user --unit everygarf.service --pager-end`

# Disclaimer

This project has no connection to *Garfield* or *Paws, Inc*. 
If you have any issues or concerns, please [create a GitHub issue](https://github.com/darccyy/everygarf/issues/new).

---

![Icon: Stylized Garfield Face](./image/icon.png)

