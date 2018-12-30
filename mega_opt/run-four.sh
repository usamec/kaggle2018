timeout 60  target/release/base_opt_quick --load ../outputs/run1-start.csv --save-to ../outputs/rf-1.01-1 --temp 0.0 --base-limit 5.0 --penalty 0.01 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 0 --seed 25666 &

timeout 60  target/release/base_opt_quick --load ../outputs/run1-start.csv --save-to ../outputs/rf-1.01-2 --temp 0.0 --base-limit 5.0 --penalty 0.01 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 0 --seed 26666 &

timeout 60  target/release/base_opt_quick --load ../outputs/run1-start.csv --save-to ../outputs/rf-1.01-3 --temp 0.0 --base-limit 5.0 --penalty 0.01 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 0 --seed 27666 &

timeout 60  target/release/base_opt_quick --load ../outputs/run1-start.csv --save-to ../outputs/rf-1.01-4 --temp 0.0 --base-limit 5.0 --penalty 0.01 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 0 --seed 28666 &

sleep 62;
cp ../outputs/rf-1.01-1-latest.csv ../outputs/rf-1.01.csv
time ./recombinator2 ../outputs/rf-1.01.csv ../outputs/rf-1.01-2-latest.csv ../outputs/rf-1.01.csv 0.03
time ./recombinator2 ../outputs/rf-1.01.csv ../outputs/rf-1.01-3-latest.csv ../outputs/rf-1.01.csv 0.03
time ./recombinator2 ../outputs/rf-1.01.csv ../outputs/rf-1.01-4-latest.csv ../outputs/rf-1.01.csv 0.03
