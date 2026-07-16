// A runnable Go example: build a feature matrix through the binding.
//
//	cargo build --release -p wickra-feature-store-c
//	# stage the library under bindings/go/lib/<goos>_<goarch>/ (CI does this)
//	cd examples/go && go run .
package main

import (
	"fmt"

	wickra "github.com/wickra-lib/wickra-feature-store/bindings/go"
)

const spec = `{"universe":["AAA","BBB"],"features":[` +
	`{"kind":"indicator","name":"Sma","params":[2]},` +
	`{"kind":"price","field":"close"}],` +
	`"labels":[{"kind":"forward_return","horizon":1}]}`

const cmd = `{"cmd":"build_batch","data":{` +
	`"AAA":[` +
	`{"time":1,"open":10,"high":10,"low":10,"close":10,"volume":1},` +
	`{"time":2,"open":11,"high":11,"low":11,"close":11,"volume":1},` +
	`{"time":3,"open":12,"high":12,"low":12,"close":12,"volume":1}],` +
	`"BBB":[` +
	`{"time":1,"open":20,"high":20,"low":20,"close":20,"volume":1},` +
	`{"time":2,"open":22,"high":22,"low":22,"close":22,"volume":1},` +
	`{"time":3,"open":24,"high":24,"low":24,"close":24,"volume":1}]}}`

func main() {
	store, err := wickra.New(spec)
	if err != nil {
		panic(err)
	}
	defer store.Close()

	matrix, err := store.Command(cmd)
	if err != nil {
		panic(err)
	}

	fmt.Println("wickra-feature-store", wickra.Version())
	fmt.Println(matrix)
}
