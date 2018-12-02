#!/bin/bash
if [ $# -ne 1 ]; then
  echo "Usage: $0 file"
  exit 1
fi
bin/rainbow.py $1 out/rainbow.out
gnuplot bin/rainbow.plt
