#!/bin/bash

OPTION=1
# HOST="10.108.21.59"
HOST="127.0.0.1"
PORT=11235
THREAD_NUM=32
ITER_NUM=100
REQ_NUM_ARRAY=(1 10 100 500 1000 1500 2000)

for req_num in ${REQ_NUM_ARRAY[@]}
do
    ./client --host $HOST --port $PORT --option $OPTION --thread $THREAD_NUM \
        --reqnum $req_num --iter $ITER_NUM
done


# (@arg host: --host +takes_value "host on which the tutorial server listens")
#         (@arg port: --port +takes_value "port on which the tutorial server listens")
#         (@arg iter: --iter +takes_value "Iteration Numbers")
#         (@arg thread: --thread +takes_value "Thread Numbers")
#         (@arg reqnum: --reqnum +takes_value "Request Numbers")
#         (@arg bufsize: --bufsize +takes_value "Buffer Size in kB")