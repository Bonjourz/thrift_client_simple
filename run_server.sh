#!/bin/bash

if [ $# == 3 ]; then
    ./server --host $1 --port $2 --worker $3
else
    echo "args num: $#"
    echo "Usage: ./server --host [host] --port [port] --worker[worker_num]"
fi
