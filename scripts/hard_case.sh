OUTPUT_DIR="./hard_cases"

rm -r "${OUTPUT_DIR}"
mkdir "${OUTPUT_DIR}"

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_30k.txt" \
  --coeff 0.01 \
  -n 30000 \
  --value-upper-bound 80000 \
  --weight-upper-bound 120000 \
  --correlation Some

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_20k.txt" \
  --coeff 0.01 \
  -n 20000 \
  --value-upper-bound 80000 \
  --weight-upper-bound 120000 \
  --correlation Some

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_10k.txt" \
  --coeff 0.01 \
  -n 10000 \
  --value-upper-bound 80000 \
  --weight-upper-bound 120000 \
  --correlation Some

cargo run --release -- generate \
  -o "${OUTPUT_DIR}/hard_1k.txt" \
  --coeff 0.01 \
  -n 1000 \
  --value-upper-bound 80000 \
  --weight-upper-bound 120000 \
  --correlation Some
