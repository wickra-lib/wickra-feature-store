package wickra

import (
	"encoding/json"
	"strings"
	"testing"
)

const spec = `{"universe":["AAA"],` +
	`"features":[{"kind":"indicator","name":"Sma","params":[2]},{"kind":"price","field":"close"}],` +
	`"labels":[{"kind":"forward_return","horizon":1}]}`

func candle(ts int, close float64) map[string]float64 {
	return map[string]float64{
		"ts": float64(ts), "open": close, "high": close,
		"low": close, "close": close, "volume": 1.0,
	}
}

func candles() []map[string]float64 {
	return []map[string]float64{candle(0, 100.0), candle(1, 110.0), candle(2, 121.0)}
}

func buildBatchCmd() string {
	cmd, _ := json.Marshal(map[string]any{
		"cmd":  "build_batch",
		"data": map[string]any{"AAA": candles()},
	})
	return string(cmd)
}

func TestVersion(t *testing.T) {
	if Version() == "" {
		t.Fatal("empty version")
	}
}

func TestBuildBatchMatrix(t *testing.T) {
	s, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer s.Close()
	raw, err := s.Command(buildBatchCmd())
	if err != nil {
		t.Fatal(err)
	}
	var matrix struct {
		Columns []string            `json:"columns"`
		Index   []json.RawMessage   `json:"index"`
		Data    [][]json.RawMessage `json:"data"`
		Rows    int                 `json:"rows"`
	}
	if err := json.Unmarshal([]byte(raw), &matrix); err != nil {
		t.Fatal(err)
	}
	want := []string{"Sma(2)", "price.close", "fwd_return(1)"}
	if len(matrix.Columns) != len(want) {
		t.Fatalf("expected columns %v, got %s", want, raw)
	}
	for i, c := range want {
		if matrix.Columns[i] != c {
			t.Fatalf("expected columns %v, got %s", want, raw)
		}
	}
	if matrix.Rows != len(matrix.Data) {
		t.Fatalf("rows %d != len(data) %d", matrix.Rows, len(matrix.Data))
	}
}

func TestInvalidSpecIsError(t *testing.T) {
	if _, err := New("{ not valid json"); err == nil {
		t.Fatal("expected an error for an invalid spec")
	}
}

func TestUnknownCommandIsInBandError(t *testing.T) {
	s, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer s.Close()
	raw, err := s.Command(`{"cmd":"nope"}`)
	if err != nil {
		t.Fatalf("unexpected hard error: %v", err)
	}
	if !strings.Contains(raw, `"ok":false`) {
		t.Fatalf("expected an in-band error, got: %s", raw)
	}
}
