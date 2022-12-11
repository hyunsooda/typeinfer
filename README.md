
- Run
The command below run the current implementation of javascript type inference with an example at `example/example.js`
`cargo run`
It reports like below:
```
[Detected cmp violation] Undefined == Number
  if (a == 10) { (example/example.js:3:4)
[Detected arithmetic violation] Undefined + Number
    if (a+10 < 30) { (example/example.js:4:6)
[Detected arithmetic violation] Bool + Bool
      b = false + true; (example/example.js:6:4)
```

- Test
`cargo test``


- TODO
    - Prepare function paramater
    - (Done) More accurate precision (ovewrite types in control-flow statements)
    - Implement call graph
    - (3) Consider to debloat control flow statements (for-loop, switch, etc)
    - Find entry point and run analysis from there
    - (Done) Add scope-level annotation
    - (1) The current implementation is object-insensitive. Consider to change as object-sensitive
    - (2) Consider array
    - Differenciate the semantic of `let` and `var`
    - (0) Seperate pre-analysis (debloat crate) as another crate
