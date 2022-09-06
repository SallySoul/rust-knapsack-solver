OUTPUT_DIR="/Users/russell/projects/courses/cse5707/rust-knapsack-solver/temp/hard_cases"

rm -r "${OUTPUT_DIR}"
mkdir "${OUTPUT_DIR}"

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_500k.txt" \
  --coeff 0.01 \
  -n 500000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_100k.txt" \
  --coeff 0.01 \
  -n 100000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_10k.txt" \
  --coeff 0.01 \
  -n 10000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_1k.txt" \
  --coeff 0.01 \
  -n 1000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some
