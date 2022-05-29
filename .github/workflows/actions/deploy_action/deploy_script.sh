#!/bin/bash
cd $GITHUB_WORKSPACE

# Expected environment variables: AWS_SECRET, AWS_SECRET_KEY, LOL_API_KEY, DISCORD_PUBLIC_KEY
serverless plugin install -n serverless-rust
serverless deploy --region=eu-west-1
