cargo build --release && \
  /usr/bin/time -l ./target/release/rust-knapsack-solver \
  solve \
  -s Minknap \
  -i temp/hard_cases_2/hard_50k.txt
