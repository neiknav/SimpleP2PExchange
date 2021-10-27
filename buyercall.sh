#!/bin/bash
set -e

near call $DEPLOY_ID --accountId=$BUYER_ID --gas=300000000000000 "$@"
