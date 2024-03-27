name := "everygarf"

# Check, build, install, publish
release:
    just install-deps &&\
    just check &&\
    cargo install --path . &&\
    cargo build --release &&\
    just cross &&\
    just dist &&\
    just publish

# Install build dependencies
install-deps:
    cargo install cross &&\
    rustup component add clippy

# Test, format, and lint
check:
    cargo test &&\
    cargo fmt &&\
    cargo clippy

# Cross compile to x86 Windows
cross:
    cross build --release --target x86_64-pc-windows-gnu

# Move release builds to dist directory
dist:
    [ -e dist/ ] && rm -rf dist/ ;\
    mkdir dist/ &&\
    mv target/release/{{name}} dist/ &&\
    mv target/x86_64-pc-windows-gnu/release/{{name}}.exe dist/ &&\
    ls dist

# Publish to Crates.io
publish:
    cargo publish

