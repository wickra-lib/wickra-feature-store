/* R .Call glue for the wickra-feature-store C ABI hub. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>
#include <stddef.h>
#include "wickra_feature_store.h"

/* --- handle lifetime ----------------------------------------------------- */

static void wkfeaturestore_finalize(SEXP ext) {
    WickraFeatureStore *h = (WickraFeatureStore *)R_ExternalPtrAddr(ext);
    if (h) {
        wickra_feature_store_free(h);
    }
    R_ClearExternalPtr(ext);
}

static WickraFeatureStore *handle_of(SEXP ext) {
    WickraFeatureStore *h = (WickraFeatureStore *)R_ExternalPtrAddr(ext);
    if (!h) {
        Rf_error("wickra-feature-store: handle is closed");
    }
    return h;
}

/* --- exported .Call entries ---------------------------------------------- */

SEXP wkfeaturestore_version(void) {
    return Rf_mkString(wickra_feature_store_version());
}

SEXP wkfeaturestore_new(SEXP spec_json) {
    const char *spec = CHAR(STRING_ELT(spec_json, 0));
    WickraFeatureStore *h = wickra_feature_store_new(spec);
    if (!h) {
        Rf_error("wickra-feature-store: invalid spec");
    }
    SEXP ext = PROTECT(R_MakeExternalPtr(h, R_NilValue, R_NilValue));
    R_RegisterCFinalizerEx(ext, wkfeaturestore_finalize, TRUE);
    UNPROTECT(1);
    return ext;
}

SEXP wkfeaturestore_command(SEXP ext, SEXP cmd_json) {
    WickraFeatureStore *h = handle_of(ext);
    const char *cmd = CHAR(STRING_ELT(cmd_json, 0));

    /* Length-out protocol: learn the length, then read into a caller buffer.
       Domain errors come back in-band as {"ok":false,...} JSON, not a negative
       code; only unusable arguments / a caught panic return < 0. */
    int len = wickra_feature_store_command(h, cmd, NULL, 0);
    if (len < 0) {
        Rf_error("wickra-feature-store: command failed (code %d)", len);
    }
    char *buf = (char *)R_alloc((size_t)len + 1, 1);
    wickra_feature_store_command(h, cmd, buf, (size_t)len + 1);
    return Rf_mkString(buf);
}

/* --- registration -------------------------------------------------------- */

static const R_CallMethodDef CallEntries[] = {
    {"wkfeaturestore_version", (DL_FUNC)&wkfeaturestore_version, 0},
    {"wkfeaturestore_new", (DL_FUNC)&wkfeaturestore_new, 1},
    {"wkfeaturestore_command", (DL_FUNC)&wkfeaturestore_command, 2},
    {NULL, NULL, 0}};

void R_init_wickrafeaturestore(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
