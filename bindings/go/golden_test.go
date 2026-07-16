package wickra

// The cross-language golden invariant seen from Go: the same command yields
// byte-identical output across calls, and streaming a spec bar-by-bar matches
// the batch build. The response bytes are what every other binding produces too,
// because the whole feature fold lives once in the Rust core and this binding
// forwards its JSON verbatim.

import (
	"encoding/json"
	"testing"
)

func TestBuildBatchByteIdenticalAcrossCalls(t *testing.T) {
	a, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer a.Close()
	b, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer b.Close()

	ra, err := a.Command(buildBatchCmd())
	if err != nil {
		t.Fatal(err)
	}
	rb, err := b.Command(buildBatchCmd())
	if err != nil {
		t.Fatal(err)
	}
	if ra != rb {
		t.Fatalf("expected byte-identical output, got:\n a: %s\n b: %s", ra, rb)
	}
}

func TestStreamingMatchesBatch(t *testing.T) {
	batchStore, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer batchStore.Close()
	batch, err := batchStore.Command(buildBatchCmd())
	if err != nil {
		t.Fatal(err)
	}

	streamed, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer streamed.Close()
	for _, c := range candles() {
		push, _ := json.Marshal(map[string]any{"cmd": "push", "symbol": "AAA", "candle": c})
		if _, err := streamed.Command(string(push)); err != nil {
			t.Fatal(err)
		}
	}
	built, err := streamed.Command(`{"cmd":"build"}`)
	if err != nil {
		t.Fatal(err)
	}
	if built != batch {
		t.Fatalf("streaming build must match batch, got:\n stream: %s\n batch:  %s", built, batch)
	}
}
