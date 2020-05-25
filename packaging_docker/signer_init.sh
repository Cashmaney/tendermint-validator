#!/usr/bin/env bash

set -e

sed -i 's/x.x.x.x/'"$NODE_ADDRESS"'/g' /root/.signer/config/config.toml

signer --import /root/priv_validator_key.json --password "$PASSWORD"

signer
