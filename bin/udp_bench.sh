#!/bin/sh
PWD=$(pwd)
outPre="$PWD/results"
log="$outPre/log"
rsDir="$PWD/rust"
addr="${HOST_IP:-127.0.0.1}"

header()
{
    echo $@
    (
        echo "\n========================================================="
        echo $@
        echo "=========================================================\n"
    ) >> $log
}


mkdir -p $outPre
: > $log


for i in "100 100B" "1000 1KB" "10000 10KB" "100000 100KB" "500000 500KB" "1000000 1MB"
do
    set -- $i
    size=$1
    sizeStr=$2
    echo "${sizeStr}"
    
    cd $rsDir
    header "rust"
    (
        set -x
        cargo run --release -- -x 1 -t rsmem -n 1 -d $size -r 1 -k lol -s $addr
        cargo run --release -- -x 1 -t rsmem -n 10000 -d 0 -r 0 -k lol -o ${outPre}/${sizeStr}_TCP_ -s $addr
        cargo run --release -- -x 1 -t rsmem -n 10000 -d 0 -r 0 -k lol -o ${outPre}/${sizeStr}_UDP_ -s $addr -u 11311
    ) >> $log 2>&1
done