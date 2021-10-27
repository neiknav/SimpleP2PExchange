#!/bin/bash
set -e

near call $DEPLOY_ID --accountId=$SELLER_ID --gas=300000000000000 "$@"
