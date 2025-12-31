# ZapPool Backend

Backend components for ZapPool proxy pool; zappool.org

A pool for home miners with daily payouts.


## Summary

Pool supporting Bitaxes and other small homeminers,
with daily payouts, integrated with Nostr & cashu,
using Ocean as an upstream pool (proxypool & motherpool).

Use your BitAxe to generate daily sats ready for zapping!


## Components

- [workstat](workstat/README.md): API to store submitted workshares.
- [ocean-gateway](ocean-gateway/README.md): Modified datum-gateway. Miners connect to this component, it supplies them with work, and completed work is forwarded to the Ocean pool.
- [oceanmgr](oceanmgr/README.md): Retrieve and save info from Ocean (rewards, payments)
- [paycalc-rs](paycalc-rs/README.md): Calculate payments, from work items and block rewards.
- [payer]: Execure payments. Separate library, but executed with paycalc-rs.


## Roadmap

- User Dashboard with Nostr login
- Beefed-up Dashboard
- User Settings


## Versions

### v0.4 - Zap!: 20251231
Important changes:
- Payout as Nostr Zaps
- Payer service rewritten in Rust (opensourced, moved to zappool-backend)
- User-specific Dashboard
- Backup
- Switch to 8-blocks reward span (to be in sync with Ocean)
- Workstat API protected by secret
- Extend web, faq
- Misc improvements

### v0.3 - Beta:  20251017
Important changes:
- Web page, Dashboard
- New environment
- Beta version
Features:
- Proxy work (accept work, forward to mother pool), proxy rewards (receive reward, send out to users, bridge if needed). Users specify Nostr npub. Domain, website, basic dashboard, basic user dashboard. No config.
- Components, database. No backup, dashboard, monitoring, redundancy.

### v0.2 - Alpha:  20251006
Important changes:
- Miner ID is an npub now, payout is done to LN Address from Nostr profile
- First alpha users
Features:
- Proxy work (accept work, forward to mother pool), proxy rewards (receive reward, send out to users, bridge if needed). Users specify Nostr npub. No domain, no website, no dashboard. No user dashboard. no config.
- Components, database. No backup, dashboard, monitoring, redundancy.

### v0.1 - Prototype1:  20250926
- Proxy work (accept work, forward to mother pool), proxy rewards (receive reward, send out to users, bridge if needed). Users specify LN address. No domain, no website, no dashboard. No user dashboard. no config.
- Components, database. No backup, dashboard, monitoring, redundancy.

