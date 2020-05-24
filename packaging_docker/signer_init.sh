#!/usr/bin/env bash

perl -i -pe 's/"x.x.x.x"/"'"$NODE_ADDRESS"'"/g' ~/.signer/config/config.toml

/bin/bash
