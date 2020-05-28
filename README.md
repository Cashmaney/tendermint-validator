# Tendermint Validator

A lightweight single key tendermint validator for sentry nodes. Based on https://gitlab.com/polychainlabs/tendermint-validator

With modifications to work with tendermint 0.33.x, inside SGX. 

While this is still a WIP, it has been running on The Secret Network mainnet for the past week with 0 downtime on an Azure Confidential Compute machine - 
https://explorer.cashmaney.com/validator/6F23B77EE70DE196515423C2038659923C94E397

## Design

A lightweight alternative to using a full node instance for validating blocks. The validator is able to connect to any number of sentry nodes and will sign blocks provided by the nodes. 

The validator maintains a watermark file to protect against double signing (_outside the enclave for now_)

## Pre-requisites

Before starting, please make sure to fully understand node and validator requirements and operation for your particular network and chain.

You will also need an SGX-capable machine

## Setup SGX

Setting up SGX is outside the scope of this document. This package has been configured to work with version 2.9.1. You can
refer to pages such as [this](https://github.com/enigmampc/EnigmaBlockchain/blob/master/docs/dev/setup-sgx.md) for installation 
instructions.

## Software Mode

If you don't have an SGX-capable machine at the ready, you can still test out this validator for yourself in software mode.
*This will not protect your keys in a safe manner and you should never use this mode for anything other than testing*

Run the prebuilt docker image:
`docker run -it cashmaney/sgx_signer_sw:latest`

Then setup using the steps [here](#configure-validator-instance)

## Docker Setup

You can use the handy docker-compose.yaml file in this repository to quickstart (and avoid having to worry about aesm) your validator.

1. Copy the `docker-compose.yaml` file to your local machine.

2. (optional) Automatically configure a remote node address by setting the `NODE_ADDRESS` environment variable:

    ```export NODE_ADDRESS=x.x.x.x```

3. Start the node using

    `docker-compose up`

4. Perform the steps [here](#configure-validator-instance)

_If you are not using a cloud-provider VM you should replace `/dev/sgx` with `/dev/isgx` in the docker-compose.yaml file_

## Setup Validator

**NOT AVAILABLE YET - USE DOCKER OR MANUALLY COMPILE**

Download the package: 

```bash
wget https://github.com/Cashmaney/tendermint-validator/releases/download/0.5.0/sgx-validator_0.5.0_amd64.deb
```

Unpack:

```bash
sudo dpkg -i tendermint-validator_0.5.0_amd64.deb
```

### Configure Validator Instance

You will find the default file in ``~/.signer/config/config.toml``
```toml

# The state directory stores watermarks for double signing protection.
# Each validator instance maintains a watermark.
state_dir = "/path/to/state/dir"

# The network chain id for your p2p nodes
chain_id = "chain-id-here"

# Configure any number of p2p network nodes.
# We recommend at least 2 nodes for redundancy.
[[node]]
address = "tcp://<node-a ip:1234"

[[node]]
address = "tcp://<node-b ip>:1234"
```

By default, the signer will generate a random private key. If you wish to use this key, you can export the validator address using
`signer --validator-address <chain-id>`

You will also need to import your private key into the SGX enclave. To do this, run:

`signer --import /path/to/key/file`

Then choose a password to protect this key.

#### Key file format
You can import one of two formats:

1. Cosmos private validator keys - a `priv_validator_key.json` file [example](#example-json-file)
2. Base64 encoding of the private key (decrypted) [example](#example-base64-key)

Import and exporting of encrypted private keys is a future improvement.

##### Example Json file
```json
{
  "address": "6F23B77EE70DE196515423C2038659923C94E397",
  "pub_key": {
    "type": "tendermint/PubKeyEd25519",
    "value": "49uQVczw4fFyIDoWknVsV0NOEcWAyfgxcT56QnQZDqo="
  },
  "priv_key": {
    "type": "tendermint/PrivKeyEd25519",
    "value": "j3Tncxe2hyCIJjRhewkFeFr9kmox741YothJCGBa4Kjj25BVzPDh8XIgOhaSdWxXQ04RxYDJ+DFxPnpCdBkOqg=="
  }
}
``` 

##### Example base64 key
```text
j3Tncxe2hyCIJjRhewkFeFr9kmox741YothJCGBa4Kjj25BVzPDh8XIgOhaSdWxXQ04RxYDJ+DFxPnpCdBkOqg==
```

#### Checking configured key (doesn't work yet :( )

Check your key has been properly imported with 

`signer --validator-address <chain-id>` 

#### Exporting a private key

To export a key, simple use the command

`signer --export <path/to/destination>`

And enter the password you configured when importing the key

## Configure p2p network nodes

Validators are not directly connected to the p2p network nor do they store chain and application state. They rely on nodes to receive blocks from the p2p network, make signing requests, and relay the signed blocks back to the p2p network.

To make a node available as a relay for a validator, find the `priv_validator_laddr` (or equivalent) configuration item in your node's configuration file. Change this value, to accept connections on an IP address and port of your choosing.

```diff
 # TCP or UNIX socket address for Tendermint to listen on for
 # connections from an external PrivValidator process
-priv_validator_laddr = ""
+priv_validator_laddr = "tcp://192.168.0.1:25567"
```

_Full configuration and operation of your tendermint node is outside the scope of this guide. You should consult your network's documentation for node configuration._

_We recommend hosting nodes on separate and isolated infrastructure from your validator instances._

## Launch validator

Once your validator instance and node is configured, you can launch the signer.

```bash
sudo systemctl start validator-node
```

View the logs of the validator using

```bash
journalctl -f -u validator-node
```

## Building from source

### Requirements

* SGX_SDK 2.9.101.2
* Rust 04-07-2019
* Go 1.14+ 

#### Locally

If you're a brave soul that wants to build this from source outside of docker just `make` should trigger all the good stuff.

You'll end up with `./build/signer` and `./build/enclave.signed.so`.
The enclave file should go in a path it can be found, such as `/usr/lib/` or, set the enclave directory using the environment variable `ENCLAVE_DIR`

#### Inside docker

Easier than setting everything up is just to build inside a docker container.

Build using:

`docker build -t cashmaney/sgx_signer .`

You can find the base images under `./packaging_docker/`

## No Liability

As far as the law allows, this software comes as is,
without any warranty or condition, and no contributor
will be liable to anyone for any damages related to this
software or this license, under any kind of legal claim.

## References

- https://docs.tendermint.com/master/tendermint-core/validators.html
- https://hub.cosmos.network/master/validators/overview.html
