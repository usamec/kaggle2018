if (!exists("filename")) filename="diff.out"

plot filename using 1:2:3:4:5 with vectors lc rgb variable
pause -1
