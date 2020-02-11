#!/usr/bin/env zsh

help()
{
   echo ""
   echo "Usage: $0 -s CLUSTER_SIZE"
   echo -e "\t-size Launches a zeroconf AP Artillery cluster"
   exit 1
}

while getopts "s:" opt
do
   case "$opt" in
      s ) CLUSTER_SIZE="$OPTARG" ;;
      ? ) help ;;
   esac
done

if [ -z "$CLUSTER_SIZE" ]
then
   echo "Parameter expected";
   help
fi

mkdir -p deployment-tests/node_state
cd deployment-tests/node_state

for i in {1..$CLUSTER_SIZE}
do
  echo "Starting Node: $i"
  NODE_DATA_DIR="node$i"
  mkdir -p $NODE_DATA_DIR
  RUST_BACKTRACE=full RUST_LOG=debug cargo run --example cball_mdns_sd_infection $NODE_DATA_DIR &
  sleep 1
done
