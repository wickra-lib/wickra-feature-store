// Package wickra provides idiomatic Go bindings for wickra-feature-store over
// its C ABI hub: build a FeatureStore from a spec JSON, drive it with command
// JSON (set_spec, push, push_batch, build, build_batch, labels, reset, version)
// and read back the response JSON — the same protocol as the CLI and every other
// binding.
//
// The binding links the prebuilt C ABI library, staged per platform under
// ./lib/<goos>_<goarch>/, with the header vendored under ./include.
package wickra

/*
#cgo CFLAGS: -I${SRCDIR}/include
#cgo linux,amd64 LDFLAGS: -L${SRCDIR}/lib/linux_amd64 -lwickra_feature_store -Wl,-rpath,${SRCDIR}/lib/linux_amd64
#cgo linux,arm64 LDFLAGS: -L${SRCDIR}/lib/linux_arm64 -lwickra_feature_store -Wl,-rpath,${SRCDIR}/lib/linux_arm64
#cgo darwin,amd64 LDFLAGS: -L${SRCDIR}/lib/darwin_amd64 -lwickra_feature_store -Wl,-rpath,${SRCDIR}/lib/darwin_amd64
#cgo darwin,arm64 LDFLAGS: -L${SRCDIR}/lib/darwin_arm64 -lwickra_feature_store -Wl,-rpath,${SRCDIR}/lib/darwin_arm64
#cgo windows,amd64 LDFLAGS: -L${SRCDIR}/lib/windows_amd64 -l:wickra_feature_store.dll
#cgo windows,arm64 LDFLAGS: -L${SRCDIR}/lib/windows_arm64 -l:wickra_feature_store.dll
#include <stdlib.h>
#include "wickra_feature_store.h"
*/
import "C"

import (
	"fmt"
	"runtime"
	"unsafe"
)

// FeatureStore is a feature store driven by JSON commands, built from a spec.
type FeatureStore struct {
	handle *C.WickraFeatureStore
}

// New builds a feature store from a spec JSON string. It returns an error if the
// spec is null, not valid UTF-8, or not a valid spec. Call Close when done (a
// finalizer also frees it, but explicit Close is preferred).
func New(specJSON string) (*FeatureStore, error) {
	cspec := C.CString(specJSON)
	defer C.free(unsafe.Pointer(cspec))

	handle := C.wickra_feature_store_new(cspec)
	if handle == nil {
		return nil, fmt.Errorf("wickra-feature-store: invalid spec")
	}
	s := &FeatureStore{handle: handle}
	runtime.SetFinalizer(s, (*FeatureStore).Close)
	return s, nil
}

// Command applies a command JSON and returns the response JSON. It uses the C
// ABI's length-out protocol: a first call learns the length, then the response
// is read into a caller-owned buffer.
func (s *FeatureStore) Command(cmdJSON string) (string, error) {
	ccmd := C.CString(cmdJSON)
	defer C.free(unsafe.Pointer(ccmd))

	n := C.wickra_feature_store_command(s.handle, ccmd, nil, 0)
	if n < 0 {
		return "", fmt.Errorf("wickra-feature-store: command failed (code %d)", int(n))
	}
	buf := make([]byte, int(n)+1)
	C.wickra_feature_store_command(
		s.handle,
		ccmd,
		(*C.char)(unsafe.Pointer(&buf[0])),
		C.uintptr_t(len(buf)),
	)
	return string(buf[:n]), nil
}

// Close frees the store handle. Safe to call more than once.
func (s *FeatureStore) Close() {
	if s.handle != nil {
		C.wickra_feature_store_free(s.handle)
		s.handle = nil
	}
	runtime.SetFinalizer(s, nil)
}

// Version returns the library version.
func Version() string {
	return C.GoString(C.wickra_feature_store_version())
}
