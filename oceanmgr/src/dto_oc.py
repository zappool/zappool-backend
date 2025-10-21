from datetime import datetime, UTC

# Block earning: a piece of earned earning, connected to a block found
class BlockEarning:
    def __init__(self, time: int, block_hash: str, earned_sats: int, pool_fee: int):
        self.time = time
        self.block_hash = block_hash
        self.earned_sats = earned_sats
        self.pool_fee = pool_fee

    def to_string(self) -> str:
        return f"{datetime.fromtimestamp(self.time, UTC)} {self.block_hash} {self.earned_sats} {self.pool_fee}"
