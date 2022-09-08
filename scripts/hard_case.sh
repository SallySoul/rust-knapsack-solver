OUTPUT_DIR="/mnt/c/Users/blake/Documents/rust-knapsack-solver/hard_cases"

rm -r "${OUTPUT_DIR}"
mkdir "${OUTPUT_DIR}"

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_30k.txt" \
  --coeff 0.01 \
  -n 30000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_20k.txt" \
  --coeff 0.01 \
  -n 20000 \
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
