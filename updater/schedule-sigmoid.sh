trap 'kill $(jobs -p)' EXIT

do_iteration() {
  IN="--load ../outputs/iter.csv"
  OUT="--save-to ../outputs/pp"
  THREADS="--n-threads 1 --n-heavy-threads 2 --n-brute-threads 1"
  echo "Running penalty $2 for $1 sec"
  TIME=$1
  PENALTY="--penalty-threshold $2"
  echo "Current step: $1 $2" > ../outputs/step.txt
  timeout $TIME target/release/kopt2 $IN $OUT $THREADS $PENALTY
  mv ../outputs/pp-best.csv ../outputs/iter.csv
}


make
#cp ../outputs/start.csv ../outputs/iter.csv

# We want to run for about 1 day
BASE=$((1*86400/31))
INF=$((86400*100))

do_iteration $BASE 200.0
do_iteration $BASE 180.0
do_iteration $BASE 150.0
do_iteration $BASE 125.0
do_iteration $BASE 100.0
do_iteration $BASE 70.0
do_iteration $BASE 50.0
do_iteration $BASE 40.0
do_iteration $BASE 30.0
do_iteration $BASE 20.0
do_iteration $BASE 15.0
do_iteration $BASE 12.0
do_iteration $BASE 10.0
do_iteration $BASE 9.5
do_iteration $BASE 9.0
do_iteration $BASE 8.5
do_iteration $BASE 8.0
do_iteration $BASE 7.5
do_iteration $BASE 7.0
do_iteration $BASE 6.5
do_iteration $BASE 6.0
do_iteration $BASE 5.5
do_iteration $BASE 5.0
do_iteration $BASE 4.5
do_iteration $BASE 4.0
do_iteration $BASE 3.5
do_iteration $BASE 3.0
do_iteration $BASE 2.5
do_iteration $BASE 2.0
do_iteration $BASE 1.5
do_iteration $BASE 1.0
do_iteration $BASE 0.5
do_iteration $INF 0.000001
