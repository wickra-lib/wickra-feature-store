## Plain-R tests for the wickra-feature-store R binding (no testthat dependency).
## Mirrors the Rust/Python/Node/Go/C#/Java tests and doubles as the completeness
## guard: it exercises the full public surface (version + new + command).

library(wickrafeaturestore)

spec <- paste0(
  '{"universe":["AAA"],',
  '"features":[{"kind":"indicator","name":"Sma","params":[2]},',
  '{"kind":"price","field":"close"}],',
  '"labels":[{"kind":"forward_return","horizon":1}]}'
)

candle <- function(ts, close) {
  paste0(
    '{"ts":', ts, ',"open":', close, ',"high":', close,
    ',"low":', close, ',"close":', close, ',"volume":1.0}'
  )
}

candles <- function() {
  paste0("[", candle(0, 100.0), ",", candle(1, 110.0), ",", candle(2, 121.0), "]")
}

build_batch_cmd <- function() {
  paste0('{"cmd":"build_batch","data":{"AAA":', candles(), "}}")
}

## version
stopifnot(nzchar(wkfeaturestore_version()))

## build_batch returns the expected feature matrix
store <- wkfeaturestore_new(spec)
matrix <- wkfeaturestore_command(store, build_batch_cmd())
stopifnot(grepl('"Sma(2)"', matrix, fixed = TRUE))
stopifnot(grepl('"price.close"', matrix, fixed = TRUE))
stopifnot(grepl('"fwd_return(1)"', matrix, fixed = TRUE))
stopifnot(grepl('"rows":3', matrix, fixed = TRUE))

## build_batch is byte-identical across stores (the cross-language golden core)
store2 <- wkfeaturestore_new(spec)
matrix2 <- wkfeaturestore_command(store2, build_batch_cmd())
stopifnot(identical(matrix, matrix2))

## an invalid spec is a hard error at construction
err <- tryCatch(wkfeaturestore_new("{ not valid json"), error = function(e) e)
stopifnot(inherits(err, "error"))

## an unknown command is an in-band error, not a hard error
inband <- wkfeaturestore_command(store, '{"cmd":"nope"}')
stopifnot(grepl('"ok":false', inband, fixed = TRUE))

## streaming a spec bar-by-bar matches the batch build
streamed <- wkfeaturestore_new(spec)
for (i in 0:2) {
  close <- c(100.0, 110.0, 121.0)[i + 1]
  push <- paste0('{"cmd":"push","symbol":"AAA","candle":', candle(i, close), "}")
  invisible(wkfeaturestore_command(streamed, push))
}
built <- wkfeaturestore_command(streamed, '{"cmd":"build"}')
stopifnot(identical(built, matrix))

cat("wickra-feature-store R tests passed\n")
