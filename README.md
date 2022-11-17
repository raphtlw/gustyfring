# gus

Telegram bot which provides a group chat with an "L" scoreboard, allowing members to give L's to other members of the group.

## Develop

Run

```shell
cargo run
```

Automatically reload with changes

```shell
cargo watch --exec run
```

### Development notes

Access the database

```shell
sqlite3 ~/Library/Application\ Support/gustyfring/db.sqlite3
```

## Release

Build for production

```shell
cargo build --release
```
