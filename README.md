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

