# Workstat

Keeps track of work shares, exposes a simple API to record accpeted work shares.
Intended to be used with ocean-gateway (datum-gateway-hooked).


## Setup

```
cp env-sample .env
```

Set `DB_DIR` in `.env` (e.g. to "./data").

```
cargo build
./target/debug/__workstat_setup_db
mv ./data/_new_workstat.db ./data/workstat.db
```

## Startup

```
./target/debug/workstat
```

## Test

```
curl http://localhost:5004/api/ping -H "Content-Type: application/json"
curl http://localhost:5004/api/work-count -H "Content-Type: application/json"
curl -X POST http://localhost:5004/api/work-insert -H "Content-Type: application/json" -d '{"uname_o": "user1", "uname_u": "upstream1", "tdiff": 100, "sec": "secret_value", "pool": 1}'

./target/debug/workstat_list
```

