#!/usr/bin/env sh

cargo build --release 
tests=`ls test_assets`
rm report.csv s_report.csv formated_report
for t in $tests
do
  echo "Running test: ${t}"
  /usr/bin/time -o /tmp/time \
    ./target/release/rust-knapsack-solver \
    solve -n -i test_assets/$t > /tmp/log
  time=`rg '([\d.]+) real' /tmp/time -o -r '$1' -N`
  states=`rg 'final i.*states_explored: ([\d]+)' /tmp/log -o -r '$1' -N`
  mem=`rg 'final i.*mem_used: ([\d.]+ ..)' /tmp/log -o -r '$1' -N`
  name=`basename -s '.txt' $t`
  echo "${name}, ${time}s, ${states}, ${mem}" >> report.csv
done

(echo "Name, Time, States Explored, Mem Used" && sort -k 3 -n report.csv) >> s_report.csv
column -ts, s_report.csv > formated_report.txt
