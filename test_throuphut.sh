#!/bin/bash

OPTION=1    # For throughput
HOST="10.108.21.58"
#HOST="127.0.0.1"
PORT=11235
THREAD_NUM=32
ITER_NUM=5
BUF_SIZE_IN_KB_ARRAY=(1 4 8 16 32 64 128 256 512 1024 4096)
REQ_NUM=500

for buf_size in ${BUF_SIZE_IN_KB_ARRAY[@]}
do
    ./client --host $HOST --port $PORT --option $OPTION --thread $THREAD_NUM \
        --reqnum $REQ_NUM --bufsize $buf_size --iter $ITER_NUM
done
