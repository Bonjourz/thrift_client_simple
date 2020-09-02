#!/bin/bash

if [ $# == 4 ]; then
    ./client --host $1 --port $2 --iter $3 --thread $4
else
    echo "Usage: ./client --host [host] --port [port] --iter[iter_num] --thread[thread_num]"
fi