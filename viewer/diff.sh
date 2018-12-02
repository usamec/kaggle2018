#!/bin/bash
if [ $# -ne 2 ]; then
  echo "Usage $0 file1 file2"
  exit 1
fi
bin/diff.py $1 $2 out/diff.out
gnuplot bin/diff.plt
