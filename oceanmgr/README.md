Retrieve and save info from Ocean (rewards, payments)

URL: https://ocean.xyz/stats/<ocean_address>

Data:

Block earning, for each block:
- Time (of the block)
- Block hash
- Earning in that block
- Pool fee

Snapshot:
- Estimated Rewards In Window
- Unpaid Earnings
- Lifetime Earnings


## Setup

```
sudo apt-get install python3-full
python3 -m venv venv
./venv/bin/pip install -r oceanmgr/requirements.txt
#sudo apt-get install python3-dotenv
cp env-sample .env
```

Set `DB_DIR` in `.env` (e.g. to "./data").

```
./venv/bin/python oceanmgr/src/__setup_db.py

mv ./data/_new_ocean.db ./data/ocean.db
```


## Startup

```
./venv/bin/python oceanmgr/src/get_earnings_main.py
```


## Debug

```
./venv/bin/python oceanmgr/src/print_earnings_main.py
```

```
curl -X POST https://ocean.xyz/data/csv/<address>/earnings
```
