#!/bin/bash
if [ $# -ne 1 ]; then
  echo "Usage: $0 file"
  exit 1
fi
./rainbow.py $1 rainbow.out
gnuplot -e 'filename="rainbow.out"' plot.plt
