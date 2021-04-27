# eir

[![GitHub Action](https://img.shields.io/github/workflow/status/raviqqe/eir/test?style=flat-square)](https://github.com/raviqqe/eir/actions?query=workflow%3Atest)
[![Codecov](https://img.shields.io/codecov/c/github/raviqqe/eir.svg?style=flat-square)](https://codecov.io/gh/raviqqe/eir)
[![License](https://img.shields.io/github/license/raviqqe/eir.svg?style=flat-square)](LICENSE)

`eir` is a structurally-typed strict functional core language supposed to be used as a target language for high-level strict functional programming languages.

This repository consists of two crates of `eir` and `eir-llvm`. The former is to construct intermediate representation (IR) of `eir` going through type check and other validation and the latter is to compile it into LLVM IR bitcode.

## Install

In your `Cargo.toml`,

```
eir = { git = "https://github.com/raviqqe/eir", branch = "master" }
eir-llvm = { git = "https://github.com/raviqqe/eir", branch = "master" }
```

## Features

- Inference of closure environment types
- Partial application
- Bit cast
- Lazy evaluation

### Ones not supported...

- Type inference
  - The IR needs to be fully-typed already.
- Generics
- Garbage collection
  - Bring your own GC.

## Type system

- Functions
- Algebraic data types
  - Constructors are boxed or unboxed explicitly.
- Primitives
  - 8-bit integer
  - 32-bit integer
  - 64-bit integer
  - 32-bit floating point number
  - 64-bit floating point number
  - Pointer

### Binary representation of ADTs

- Tags are pointer-sized integers.
- Constructor payloads boxed or unboxed contain their elements.

#### Single constructor with no payload

- Empty data

#### Single constructor with payload

| (payload size) |
| -------------- |
| payload        |

#### Multiple constructors with no payload

| (pointer size) |
| -------------- |
| tag            |

#### Multiple constructors with payload

| (pointer size) | (max payload size) |
| -------------- | ------------------ |
| tag            | payload            |

## Examples

```rust
let algebraic_type = eir::types::Algebraic::new(vec![eir::types::Constructor::boxed(vec![
    eir::types::Primitive::Float64.into(),
])]);

let bitcode = eir_llvm::compile(
    &eir::ir::Module::new(
        vec![],
        vec![eir::ir::FunctionDefinition::new(
            "f",
            vec![eir::ir::Argument::new("x", eir::types::Primitive::Float64)],
            eir::ir::ConstructorApplication::new(
                eir::ir::Constructor::boxed(algebraic_type.clone(), 0),
                vec![eir::ir::Variable("x").into()],
            ),
            algebraic_type,
        )
        .into()],
    )
    .unwrap(),
    &CompileConfiguration::new(None, None),
)?;
```

## License

[Apache 2.0](LICENSE)
