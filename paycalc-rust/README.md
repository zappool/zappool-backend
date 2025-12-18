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
cd paycalc-rust && cargo build && cd ..
./paycalc-rust/target/debug/__setup_db
mv ./data/_new_paycalc.db ./data/paycalc.db
```

## Startup

```
./paycalc-rust/target/debug/main
```

## Test

```
./paycalc-rust/target/debug/main_list_work
./paycalc-rust/target/debug/main_dashboard
cd paycalc-rust && cargo test && cd ..
```

