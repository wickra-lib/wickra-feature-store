# A runnable R example: build a feature matrix through the binding.
#
#   cargo build -p wickra-feature-store-c --release
#   export WKFS_INC="$PWD/bindings/c/include"
#   export WKFS_LIB="$PWD/target/release"
#   export LD_LIBRARY_PATH="$WKFS_LIB:$LD_LIBRARY_PATH"   # PATH on Windows
#   R CMD INSTALL bindings/r
#   Rscript examples/r/build_features.R

library(wickrafeaturestore)

spec <- paste0(
  '{"universe":["AAA","BBB"],"features":[',
  '{"kind":"indicator","name":"Sma","params":[2]},',
  '{"kind":"price","field":"close"}],',
  '"labels":[{"kind":"forward_return","horizon":1}]}'
)

candle <- function(time, close) {
  paste0(
    '{"time":', time, ',"open":', close, ',"high":', close,
    ',"low":', close, ',"close":', close, ',"volume":1}'
  )
}

store <- wkfeaturestore_new(spec)
cmd <- paste0(
  '{"cmd":"build_batch","data":{',
  '"AAA":[', candle(1, 10), ",", candle(2, 11), ",", candle(3, 12), "],",
  '"BBB":[', candle(1, 20), ",", candle(2, 22), ",", candle(3, 24), "]}}"
)
response <- wkfeaturestore_command(store, cmd)

cat("wickra-feature-store", wkfeaturestore_version(), "\n")
cat(response, "\n")
