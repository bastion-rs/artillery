#!/usr/bin/env bash

set -eo pipefail

len=$(($#-1))
export CHAIN_LEN=$len
for i in `seq 0 $(($#-1))`
do
    echo $i
    target/debug/examples/craq_node server 0 $i $* &
    export PID$i=$!
    echo ${PID}$i
done
