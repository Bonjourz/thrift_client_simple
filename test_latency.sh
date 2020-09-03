#!/bin/bash

OPTION=2    # For Latency
HOST="10.108.21.58"
#HOST="127.0.0.1"
PORT=11235
THREAD_NUM_ARRAY=(1 2 4 8 16 24 32 40 48 56 60)
ITER_NUM=5
BUF_SIZE_IN_KB_ARRAY=4
REQ_NUM=500

for thread_num in ${THREAD_NUM_ARRAY[@]}
do
    ./client --host $HOST --port $PORT --option $OPTION --thread $thread_num \
        --reqnum $REQ_NUM --bufsize $buf_size --iter $ITER_NUM
done
