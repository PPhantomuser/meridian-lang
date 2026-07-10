#!/bin/bash
~/.cargo/bin/cargo run --bin merid -- run echo_server.merid > server.log 2>&1 &
SERVER_PID=$!
sleep 2
echo "Hello TCP!" | nc 127.0.0.1 8081
sleep 1
cat server.log
kill $SERVER_PID
