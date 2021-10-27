#!/bin/bash
set -e

near deploy --accountId $DEPLOY_ID --wasmFile res/simple_p2p_exchange.wasm

