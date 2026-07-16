#' The wickra-feature-store library version.
#' @return A version string.
#' @export
wkfeaturestore_version <- function() {
  .Call(C_wkfeaturestore_version)
}

#' Build a feature store from a spec JSON.
#' @param spec_json A feature-store spec JSON string.
#' @return A `wickra_feature_store` handle (an external pointer).
#' @export
wkfeaturestore_new <- function(spec_json) {
  .Call(C_wkfeaturestore_new, spec_json)
}

#' Apply a command JSON and return the resulting response JSON.
#' @param store A store handle from [wkfeaturestore_new()].
#' @param cmd_json A command JSON string.
#' @return The response as a JSON string.
#' @export
wkfeaturestore_command <- function(store, cmd_json) {
  .Call(C_wkfeaturestore_command, store, cmd_json)
}
