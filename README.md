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

Does the Rust/cargo ecosystem behave more like [`npm` + Node.js in allowing](#nodejs--npm) incompatible transitive dependency versions or is it like [`pip` + Python and does not](#python--pip)?

Spoiler: It behaves like `npm` and thank god for that. 😇

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
cargo build --release

tree target/debug
target/release
├── build
│   ├── ...
├── deps
│   ├── a-5a849d8d164ba2f5.d
│   ├── b-1132515820191c83.d
│   ├── ...
│   ├── liblog-918cd93f416d7029.rlib
│   ├── liblog-918cd93f416d7029.rmeta
│   ├── liblog-fee03072b48908b6.rlib
│   ├── liblog-fee03072b48908b6.rmeta
│   ├── ...
│   ├── log-918cd93f416d7029.d
│   ├── log-fee03072b48908b6.d
│   ├── ...
├── examples
├── incremental
├── rust-incompatible-transitive-version-example
└── rust-incompatible-transitive-version-example.d
```

Notice that cargo has built two versions of the `.d`, `.rmeta`, and `.rlib` files for each separate version of the log dependency. The linker will use these separate files to include separate.

> NOTE: Run `cat target/release/deps/log-*.d` to see which source files were used to generate each compiled binary file.

As an exercise, try setting SemVer _compatible_ versions of the log crate in `a/Cargo.toml` and `b/Cargo.toml` and then.

1. Run `cargo clean` to empty `target/*`
1. Run `cargo build --release` again
1. Inspect the `target/release` directory again.

You should see only a single collection of intermediate files named `*log*`.

> NOTE: Did you change `b/Cargo.toml` to a `0.4` version of log that is _lower_ than `0.4.22`? If so, you may be surprised to find only the `0.4.22` version requested by `a/Cargo.toml` was fetched and built. This is because the Cargo dependency resolver takes the liberty to use the highest SemVer compatible crate version required by another depency. I.e. `0.4.10` can be treated by Cargo as `0.4.x`.

## How does this behavior relate to other languages?

As you can see below, [Python can't handle](#python--pip) the situation we've just described above. But [Node.js can](#nodejs--npm).

### Python + `pip`

```bash
# Can actually be omitted because the install will fail and no modules will be globally anyw
python -m venv venv && source venv/bin/activate

cat <<EOF > requirements.txt
flask==2.0.0
werkzeug==1.0.0
EOF

pip install -r requirements.txt
```

```plain
Collecting flask==2.0.0 (from -r requirements.txt (line 1))
  Using cached Flask-2.0.0-py3-none-any.whl.metadata (3.8 kB)
Collecting werkzeug==1.0.0 (from -r requirements.txt (line 2))
  Using cached Werkzeug-1.0.0-py2.py3-none-any.whl.metadata (4.7 kB)
INFO: pip is looking at multiple versions of flask to determine which version is compatible with other requirements. This could take a while.
ERROR: Cannot install -r requirements.txt (line 1) and werkzeug==1.0.0 because these package versions have conflicting dependencies.

The conflict is caused by:
    The user requested werkzeug==1.0.0
    flask 2.0.0 depends on Werkzeug>=2.0

To fix this you could try to:
1. loosen the range of package versions you've specified
2. remove package versions to allow pip to attempt to solve the dependency conflict

ERROR: ResolutionImpossible: for help visit https://pip.pypa.io/en/latest/topics/dependency-resolution/#dealing-with-dependency-conflicts
```

As far as I'm aware, this is a fundamental problem with Python packages and this scenario can't be avoided by newer package managers like [Rye](https://rye.astral.sh/) or [Poetry](https://python-poetry.org/).

### Node.js + `npm`

```bash
cat <<EOF > package.json
{
  "name": "incompatible-dependencies-example",
  "version": "1.0.0",
  "description": "Example project demonstrating npm allowing incompatible transitive dependencies",
  "main": "index.js",
  "dependencies": {
    "har-validator": "^4.2.1",
    "request": "^2.88.2"
  }
}
EOF

npm install # Note the dependency resolver exits just fune

grep har-validator package-lock.json
```

```plain
grep har-validator package-lock.json

        "har-validator": "^4.2.1",
    "node_modules/har-validator": {
      "resolved": "https://registry.npmjs.org/har-validator/-/har-validator-4.2.1.tgz",
        "har-validator": "~5.1.3",
    "node_modules/request/node_modules/har-validator": {
      "resolved": "https://registry.npmjs.org/har-validator/-/har-validator-5.1.5.tgz",

```

Notice how `npm` has resolved separate incompatible versions of the har-validator package? The former is the one we explicitly required and the later is the one request needs. Each resides in a separate location in `node_modules/` and can be accessed by our project at runtime.
