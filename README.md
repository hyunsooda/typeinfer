# typeinfer
Typeinfer is a static type inference tool that reports potential bugs by detecting type violations.
Unlike [Flow](https://github.com/facebook/flow), a static typing tool that requires developers to manually annotate types for the entire project,
Typeinfer does not require any effort in terms of type annotations or any strenuous work of preparation for type inference. It can automatically infer types without any explicit input from the developer.

- Run
Execute the `cargo run` command to inspect the contents of the `example/example.js` file. The output of the inspection will be displayed as follows.
```
[Detected cmp violation] Undefined == Number
  if (a == 10) { (example/example.js:3:4)
[Detected arithmetic violation] Undefined + Number
    if (a+10 < 30) { (example/example.js:4:6)
[Detected arithmetic violation] Bool + Bool
      b = false + true; (example/example.js:6:4)
```

- Test
`cargo test`

- TODO
    - Build an environment for a function parameter
    - Implement call graph
    - Consider to debloat control flow statements (for-loop, switch, etc)
    - Find entry point and run analysis from there
    - The current implementation is object-insensitive. Consider to change as object-sensitive
    - Consider array
    - differentiate the semantic of `let` and `var`
    - Seperate pre-analysis (debloat crate) as another crate
