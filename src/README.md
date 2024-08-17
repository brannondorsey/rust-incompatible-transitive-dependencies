# Can a Rust binary use _incompatible_ versions of the same library?

Yes it can.

## Background

As I've continued learning Rust and the cargo ecosystem, one question has continued to plague me:

> Can a rust project import multiple SemVer incompatible library crates, or will the dependency resolver reject this scenario and your code will fail to compile?

### Transitive dependency conflicts

This is a scenario which occurs mostly with [transitive dependencies](https://en.wikipedia.org/wiki/Transitive_dependency) in and can be a real headache, especially if you are using unmaintained libraries. A transitive dependency is a dependency of one of your dependencies, and you often don't get to control this version.

> A (your app) > B (your app's dependency) > C (youre dependency's dependency)

In this scenario, A and C are considered transitive dependencies.

### Is rust like `npm` or is it like `pip`?

Does the Rust/cargo ecosystem behave more like `npm` + Node.js in allowing incompatible transitive dependency versions or is it like `pip` + Python and does not? Spoiler: It behaves like `npm` and thank god for that. 😇

## Prove it

```bash
git clone https://github.com/brannondorsey/rust-incompatible-transitive-dependencies
cd rust-incompatible-transitive-dependencies/

cargo run # see example output below
```

```plain
   Compiling a v0.1.0 (/home/brannon/Documents/code/rust-incompatible-transitive-dependencies/a)
   Compiling b v0.1.0 (/home/brannon/Documents/code/rust-incompatible-transitive-dependencies/b)
   Compiling dependency-test v0.1.0 (/home/brannon/Documents/code/rust-incompatible-transitive-dependencies)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.74s
     Running `target/debug/dependency-test`
2024-08-17T23:02:02.709Z INFO  [a] logged using log@0.4.22
2024-08-17T23:02:02.709Z INFO  [b] logged using log@0.3.9
```

### Show me the code

`cargo.toml`

```toml
[package]
name = "dependency-test"
version = "0.1.0"
edition = "2021"

[dependencies]
a = { version = "0.1.0", path = "a" }
b = { version = "0.1.0", path = "b" }
simple_logger = "5.0.0"
```

`main.rs`

```rust
use a::log as log_a;
use b::log as log_b;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .init()
        .expect("Failed to initialize logger");
    log_a();
    log_b();
}
```

`a/cargo.toml`

```toml
[package]
name = "a"
version = "0.1.0"
edition = "2021"

[dependencies]
log = "0.4.22"
```

`a/src/lib.rs`

```rust
use log::info;

pub fn log() {
    info!("logged using log@0.4.22");
}
```

`b/cargo.toml`

```toml
[package]
name = "b"
version = "0.1.0"
edition = "2021"

[dependencies]
# Intentionally using an outdated version
log = "0.3.9"
```

`b/src/lib.rs`

```rust
// Required for the outdated 0.3.* version of log
#[macro_use]
extern crate log;

use log::info;

pub fn log() {
    info!("logged using log@0.3.9");
}
```

## What's happening under the hood?

```bash
cargo run --release

tree target/debug
```

```plain
❯ cargo run --release
   Compiling rust-incompatible-transitive-version-example v0.1.0 (/home/brannon/Documents/code/rust-incompatible-transitive-dependencies)
    Finished `release` profile [optimized] target(s) in 0.40s
     Running `target/release/rust-incompatible-transitive-version-example`
2024-08-17T23:15:15.548Z INFO  [a] logged using log@0.4.22
❯ ls -l target/release/rust-incompatible-transitive-version-example
-rwxrwxr-x 2 brannon brannon 484976 Aug 17 19:15 target/release/rust-incompatible-transitive-version-example

❯ cargo run --release
   Compiling rust-incompatible-transitive-version-example v0.1.0 (/home/brannon/Documents/code/rust-incompatible-transitive-dependencies)
    Finished `release` profile [optimized] target(s) in 0.41s
     Running `target/release/rust-incompatible-transitive-version-example`
2024-08-17T23:15:33.617Z INFO  [b] logged using log@0.3.9
❯ ls -l target/release/rust-incompatible-transitive-version-example
-rwxrwxr-x 2 brannon brannon 485472 Aug 17 19:15 target/release/rust-incompatible-transitive-version-example

❯ cargo run --release
   Compiling rust-incompatible-transitive-version-example v0.1.0 (/home/brannon/Documents/code/rust-incompatible-transitive-dependencies)
    Finished `release` profile [optimized] target(s) in 0.41s
     Running `target/release/rust-incompatible-transitive-version-example`
2024-08-17T23:16:03.044Z INFO  [a] logged using log@0.4.22
2024-08-17T23:16:03.044Z INFO  [b] logged using log@0.3.9
❯ ls -l target/release/rust-incompatible-transitive-version-example
-rwxrwxr-x 2 brannon brannon 485720 Aug 17 19:16 target/release/rust-incompatible-transitive-version-example
```
