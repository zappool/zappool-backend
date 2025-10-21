# Ocean Gateway

This component contains a modified version of the Datum implementation of Ocean.

The implementation is modified in two points:

- map usernames (from proxy pool to upstream pool)

- save work share information for accounting

This repo contains only a diff file.
The full implementation repo is at: https://github.com/zappool/datum-gateway-hooked


## Setup

Preequisites (for building):

```
sudo apt install cmake pkgconf libcurl4-openssl-dev libjansson-dev libsodium-dev libmicrohttpd-dev psmisc
```

(see https://github.com/zappool/datum-gateway-hooked?tab=readme-ov-file#installation for details)

```sh
cd ocean-gateway
./setup_datum_gateway.sh
```

This will download the `datum-gateway` implementation, apply the modifications, and build the project.

