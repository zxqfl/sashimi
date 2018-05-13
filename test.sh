#!/bin/sh

[ -z $1 ] && exit "please provide an argument naming the build"

REMOTE=jbfjacks@high-fructose-corn-syrup.csclub.uwaterloo.ca
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
NAME="$(date "+%y-%m-%d--%H:%m:%S")--$1"
cd $DIR/uci_wrapper

cargo build --release &&
    scp ../target/release/sashimi $REMOTE:engines/$NAME &&
    ssh -t $REMOTE "
        cd cutechess && \
             ./cutechess-cli \
                 -engine cmd=../engines/benchmark arg=--threads arg=1 \
                 -engine cmd=../engines/$NAME \
                 -each \
                     st=8 \
                     timemargin=2000 \
                     proto=uci \
                 -concurrency 30 \
                 -games 50 \
                 -resign movecount=5 score=80 \
                 -openings file=silversuite.pgn \
        | tee $NAME.out"
