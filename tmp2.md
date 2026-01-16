error[E0599]: no function or associated item named `parse_list` found for struct `sfv::Parser<'de>` in the current scope
--> /Users/bart/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pingora-core-0.4.0/src/protocols/http/compression/mod.rs:416:28
|
416 |         match sfv::Parser::parse_list(ac.as_bytes()) {
|                            ^^^^^^^^^^ function or associated item not found in `sfv::Parser<'_>`
|
note: if you're trying to build a new `sfv::Parser<'_>`, consider using `sfv::Parser::<'de>::new` which returns `sfv::Parser<'_>`
--> /Users/bart/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sfv-0.14.0/src/parser.rs:66:5
|
66 |     pub fn new(input: &'de (impl ?Sized + AsRef<[u8]>)) -> Self {
|     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: there is a method `parse` with a similar name, but with different arguments
--> /Users/bart/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sfv-0.14.0/src/parser.rs:85:5
|
85 |     pub fn parse<T: crate::FieldType>(self) -> SFVResult<T> {
|     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0277]: the trait bound `Algorithm: From<&TokenRef>` is not satisfied
--> /Users/bart/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pingora-core-0.4.0/src/protocols/http/compression/mod.rs:422:45
|
422 | ...                   let algorithm = Algorithm::from(s);
|                                       ^^^^^^^^^ unsatisfied trait bound
|
= help: the trait `From<&TokenRef>` is not implemented for `Algorithm`
but trait `From<&str>` is implemented for it
= help: for that trait implementation, expected `str`, found `TokenRef`

error[E0277]: the trait bound `Algorithm: From<&TokenRef>` is not satisfied
--> /Users/bart/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pingora-core-0.4.0/src/protocols/http/compression/mod.rs:425:43
|
425 | ...                   list.push(Algorithm::from(s));
|                                 ^^^^^^^^^ unsatisfied trait bound
|
= help: the trait `From<&TokenRef>` is not implemented for `Algorithm`
but trait `From<&str>` is implemented for it
= help: for that trait implementation, expected `str`, found `TokenRef`

    Checking crossbeam-deque v0.8.6
    Checking reqwest v0.11.27
    Checking sigchld v0.2.4
Compiling phf_macros v0.10.0
Compiling pest_generator v2.8.5
Checking tokio-native-tls v0.3.1
Checking pingora-header-serde v0.4.0
Checking sentry v0.26.0
Some errors have detailed explanations: E0277, E0599.
For more information about an error, try `rustc --explain E0277`.
error: could not compile `pingora-core` (lib) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
make: *** [rust-lint] Error 101
[11:30:30] [cost 69.632s] rm -rf target Cargo.lock && cargo clean