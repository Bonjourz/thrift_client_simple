#!/bin/bash

LISTEN_ADDR="10.108.21.59"
WORKER_NUM=32
PORT=11235

./server --host $LISTEN_ADDR --worker $WORKER_NUM --port $PORT
