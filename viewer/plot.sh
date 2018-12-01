#!/bin/bash
if [ $# -ne 2 ]; then
  echo "Usage file1 file2"
  exit 1
fi
./diff.py $1 $2 diff.out
gnuplot plot.plt
