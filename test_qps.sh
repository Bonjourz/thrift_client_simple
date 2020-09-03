#!/bin/bash

OPTION=0
HOST="10.108.21.58"
#HOST="127.0.0.1"
PORT=11235
ITER_NUM=1
REQ_NUM=5000
THREAD_NUM_ARRAY=(32) #(32 128 256 512 1024)

for thread_num in ${THREAD_NUM_ARRAY[@]}
do
    ./client --host $HOST --port $PORT --option $OPTION --thread $thread_num \
        --reqnum $REQ_NUM --iter $ITER_NUM
done


# (@arg host: --host +takes_value "host on which the tutorial server listens")
#         (@arg port: --port +takes_value "port on which the tutorial server listens")
#         (@arg iter: --iter +takes_value "Iteration Numbers")
#         (@arg thread: --thread +takes_value "Thread Numbers")
#         (@arg reqnum: --reqnum +takes_value "Request Numbers")
#         (@arg bufsize: --bufsize +takes_value "Buffer Size in kB")
