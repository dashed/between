# between

> Generate strings that can sort between two other strings.

`between` is a Rust port of the [`between2`](https://github.com/dashed/between2) npm package.

## Installation

https://crates.io/crates/between

```sh
cargo add between
```

## API

### `Between::new(chars: Vec<char>) -> Self`

Initializes `Between` with a custom set of characters.

```rust
use between::Between;

let between = Between::new(vec!['a', 'b', 'c', 'd', 'e']);
```

By default, `Between` uses the following characters to generate strings:

```
!0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~
```

### `between.between(a: String, b: String) -> Option<String>`

Generates a string that lexicographically sorts between `a` and `b`.

- `a` must be lexicographically less than `b`. In other words, `a < b`.
- `a` and `b` can be any string that does not end in the lowest character. this is like how there is only one number between 0 and 1 that ends in a "0".

```rust
let tween = between.between(String::from("a"), String::from("b"));
```

`tween` is an `Option<String>` that, if `Some`, contains a string that will sort between 'a' and 'b'.

### `between.after(a: String) -> Option<String>`

Generate a string that sorts between `a` and the highest character (`between.high()`).

The string `a` cannot begin with the highest character.

### `between.before(a: String) -> Option<String>`

Generate a string that sorts between the lowest character (`between.low()`) and `a`.

## Credit

All credit to [@dominictarr](https://github.com/dominictarr) for creating the original [`between`](https://github.com/dominictarr/between) module, which inspired this Rust port.

# License

MIT.
