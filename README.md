# cracbot

CRAC 操作证查询机器人，定时查询并把结果通过 Telegram bot api 发送到指定聊天

### Build

``` bash
cargo build --release
```

### Install

``` bash
cp target/release/cracbot ~/.local/bin/cracbot
cp cracbot.conf ~/.config/cracbot.conf
cp cracbot.service ~/.config/systemd/user/cracbot.service
systemctl --user daemon-reload
```

### Configuration

Just edit `~/.config/cracbot.conf`

### Run

``` bash
systemctl --user enable --now cracbot.service
```