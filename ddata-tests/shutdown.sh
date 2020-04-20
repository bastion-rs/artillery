for i in `seq 0 $CHAIN_LEN`
do
    a=`printenv PID$i`
    kill $a
    echo "kill" $a
    export PID$i= 
done

export CHAIN_LEN=
