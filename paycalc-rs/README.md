# Paycalc

Calculate payments, from work items and committed per-block rewards.
Execute payments.

## Setup

```
sudo apt-get install cargo
cp env-sample .env
```

Set `DB_DIR` in `.env` (e.g. to "./data").

```
cd paycalc-rs && cargo build && cd ..
./paycalc-rs/target/debug/__setup_db
mv ./data/_new_paycalc.db ./data/paycalc.db
```

## Startup

```
./paycalc-rs/target/debug/main
```

## Test

```
./paycalc-rs/target/debug/main_list_work
./paycalc-rs/target/debug/main_dashboard
cd paycalc-rs && cargo test && cd ..
```

