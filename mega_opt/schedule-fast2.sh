#bash run-four.sh

#timeout 500 target/release/base_opt_quick --load ../outputs/rf-1.01.csv --save-to ../outputs/fr6-1.01b --temp 0.0 --base-limit 3.0 --penalty 0.01 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 4 --n-heavy-threads 0 --n-weak-threads 0
timeout 1300 target/release/base_opt_quick --load ../outputs/fr6-1.01b-latest.csv --save-to ../outputs/fr6-1.01p --temp 0.0 --base-limit 3.0 --penalty 0.01 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 3

timeout 500 target/release/base_opt_quick --load ../outputs/fr6-1.01p-latest.csv --save-to ../outputs/fr6-1.02b --temp 0.0 --base-limit 3.0 --penalty 0.02 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 4 --n-heavy-threads 0 --n-weak-threads 0
timeout 700 target/release/base_opt_quick --load ../outputs/fr6-1.02b-latest.csv --save-to ../outputs/fr6-1.02p --temp 0.0 --base-limit 3.0 --penalty 0.02 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 3

timeout 500 target/release/base_opt_quick --load ../outputs/fr6-1.02p-latest.csv --save-to ../outputs/fr6-1.03b --temp 0.0 --base-limit 3.0 --penalty 0.03 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 4 --n-heavy-threads 0 --n-weak-threads 0
timeout 700 target/release/base_opt_quick --load ../outputs/fr6-1.03b-latest.csv --save-to ../outputs/fr6-1.03p --temp 0.0 --base-limit 3.0 --penalty 0.03 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 3

timeout 500 target/release/base_opt_quick --load ../outputs/fr6-1.03p-latest.csv --save-to ../outputs/fr6-1.04b --temp 0.0 --base-limit 3.0 --penalty 0.04 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 4 --n-heavy-threads 0 --n-weak-threads 0
timeout 700 target/release/base_opt_quick --load ../outputs/fr6-1.04b-latest.csv --save-to ../outputs/fr6-1.04p --temp 0.0 --base-limit 3.0 --penalty 0.04 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 3

timeout 500 target/release/base_opt_quick --load ../outputs/fr6-1.04p-latest.csv --save-to ../outputs/fr6-1.06b --temp 0.0 --base-limit 3.0 --penalty 0.06 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 4 --n-heavy-threads 0 --n-weak-threads 0
timeout 700 target/release/base_opt_quick --load ../outputs/fr6-1.06b-latest.csv --save-to ../outputs/fr6-1.06p --temp 0.0 --base-limit 3.0 --penalty 0.06 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 3

timeout 500 target/release/base_opt_quick --load ../outputs/fr6-1.06p-latest.csv --save-to ../outputs/fr6-1.08b --temp 0.0 --base-limit 3.0 --penalty 0.08 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 4 --n-heavy-threads 0 --n-weak-threads 0
timeout 700 target/release/base_opt_quick --load ../outputs/fr6-1.08b-latest.csv --save-to ../outputs/fr6-1.08p --temp 0.0 --base-limit 3.0 --penalty 0.08 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 3

timeout 500 target/release/base_opt_quick --load ../outputs/fr6-1.08p-latest.csv --save-to ../outputs/fr6-1.10b --temp 0.0 --base-limit 3.0 --penalty 0.10 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 4 --n-heavy-threads 0 --n-weak-threads 0
timeout 700 target/release/base_opt_quick --load ../outputs/fr6-1.10b-latest.csv --save-to ../outputs/fr6-1.10p --temp 0.0 --base-limit 3.0 --penalty 0.10 --cand-limit 50 --cand-file ../inputs/cities.cand.0.txt --n-threads 1 --n-heavy-threads 0 --n-weak-threads 3

