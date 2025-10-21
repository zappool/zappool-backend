from dto_oc import BlockEarning
from html_parse import key_value_pairs_from_html

from datetime import datetime, UTC
import requests

ocean_web_root_url = "https://ocean.xyz"
ocean_api_root_url = "https://ocean.xyz/data"

class EarningSnapshot:
    def __init__(self, time: int, accounted_paid: int, accounted_unpaid: int, estimated: int):
        self.time = time
        self.accounted_paid = accounted_paid
        self.accounted_unpaid = accounted_unpaid
        self.estimated = estimated

    def total_accounted(self) -> int:
        return self.accounted_paid + self.accounted_unpaid

    def total(self) -> int:
        return self.total_accounted() + self.estimated

    def to_string(self) -> str:
        line1 = f"acctd paid: {self.accounted_paid}   acctd unpaid: {self.accounted_unpaid}   estimated: {self.estimated}"
        line2 = f"total acctd {self.total_accounted()}   total {self.total()}   time {self.time} {datetime.fromtimestamp(self.time, UTC)}"
        return f"{line1}\n{line2}"

def sats_from_amount_str(amount: str) -> int:
    words = amount.split(' ')
    return int(float(words[0]) * 100_000_000)

def get_earning_snapshot(ocean_account: str) -> EarningSnapshot:
    url = f"{ocean_web_root_url}/stats/{ocean_account}"
    # print(url)

    response = requests.get(url)
    # print(response)
    if response.status_code != 200:
        raise Exception(f"Could not get earnings snapshot, status code {response.status_code}, url {url}")
    text = response.text
    # print(len(text))

    values = key_value_pairs_from_html(text)
    # print(f"Found {len(values)} key-value pairs  (from url {url})")
    # for k in values:
        # print(f"  '{k}': '{values[k]}'")

    estimated_in_window = None
    lifetime_accounted = None
    accounted_unpaid = None
    for k in values:
        k_upper = k.upper()
        if "ESTIMATED REWARDS IN WINDOW" in k_upper:
            estimated_in_window = sats_from_amount_str(values[k])
        elif "LIFETIME EARNINGS" in k_upper:
            lifetime_accounted = sats_from_amount_str(values[k])
        elif "UNPAID EARNINGS" in k_upper:
            accounted_unpaid = sats_from_amount_str(values[k])
    
    if estimated_in_window == None or lifetime_accounted == None or accounted_unpaid == None:
        raise Exception(f"ERROR: Could not obtain some needed value, {estimated_in_window} {lifetime_accounted} {accounted_unpaid}")
    
    accounted_paid = lifetime_accounted - accounted_unpaid

    now_utc = datetime.now(UTC).timestamp()

    snapshot = EarningSnapshot(now_utc, accounted_paid, accounted_unpaid, estimated_in_window)
    return snapshot


def get_block_earnings(ocean_address: str) -> list[BlockEarning]:
    url = f"{ocean_api_root_url}/csv/{ocean_address}/earnings"
    # print(url)

    response = requests.post(url)
    # print(response)
    if response.status_code != 200:
        raise Exception(f"Could not get earnings, status code {response.status_code}, url {url}")
    # print(response.text)
    lines = response.text.split('\n')
    # print(len(lines))
    lc = 0
    arr = []
    for l in lines:
        if lc > 0 and len(l) > 0:
            # print(f"({lc}) {l}")
            words = l.split(',')
            if len(words) >= 5:
                # print(words[0])
                time = int(datetime.fromisoformat(words[0]).replace(tzinfo=UTC).timestamp())
                # print(time.timestamp(), time)
                earned_sats = int(float(words[4]) * 100_000_000)
                pool_fee = 0
                if len(words) >= 6:
                    pool_fee = int(float(words[5]) * 100_000_000)
                earn_obj = BlockEarning(time, words[1], earned_sats, pool_fee)
                # print(f"{earn_obj.to_string()}")
                arr.append(earn_obj)
        lc += 1
    return arr


