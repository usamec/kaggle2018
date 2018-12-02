#!/bin/bash
if [ $# -ne 1 ]; then
  echo "Usage: $0 file"
  exit 1
fi
bin/cities.py out/cities.out
bin/penalty.py $1 out/penalty.out
gnuplot bin/penalty.plt
