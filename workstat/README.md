# Workstat

Keeps track of work shares, exposes a simple API to record accpeted work shares.
Intended to be used with ocean-gateway (datum-gateway-hooked).


## Setup

```
sudo apt-get install python3-full
python3 -m venv venv
./venv/bin/pip install -r workstat/requirements.txt
# sudo apt-get install python3-flask python3-dotenv
cp env-sample .env
```

Set `DB_DIR` in `.env` (e.g. to "./data").

```
./venv/bin/python workstat/src/__setup_db.py
mv ./data/_new_workstat.db ./data/workstat.db
```

## Startup

```
./venv/bin/python workstat/src/api.py
```

## Test

```
curl http://localhost:5004/api/ping -H "Content-Type: application/json"
curl http://localhost:5004/api/work-count -H "Content-Type: application/json"
curl -X POST http://localhost:5004/api/work-insert -H "Content-Type: application/json" -d '{"uname_o": "user1", "uname_u": "upstream1", "tdiff": 100}'

./venv/bin/python workstat/src/main_list.py
./venv/bin/python workstat/src/main_test.py
```

