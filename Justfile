default:
    @just --list

test:
    @cargo nextest run

lint:
    @cargo clippy --workspace --all-features -- -D warnings
    @cargo fmt --all -- --check

build:
    @cargo build

build-release:
    @cargo build --release

update:
    @cargo upgrade

build-release-linux:
    @cargo build --release --target=x86_64-unknown-linux-musl
    @strip target/x86_64-unknown-linux-musl/release/tenere

build-release-macos:
    @cargo build --release --target=x86_64-apple-darwin
    @cargo build --release --target=aarch64-apple-darwin
