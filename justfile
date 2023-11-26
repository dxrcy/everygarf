name := "everygarf"

# Run program as debug
run:
  cargo run

# Install program to system
install:
  cargo install --path .

# Cross compile to x86 Windows
cross:
  cargo install cross &&\
  cross build --release --target x86_64-pc-windows-gnu &&\
  cd target/x86_64-pc-windows-gnu/release/

# Build program with all targets: debug, release (installed), and x86 Windows release
build-all:
  cargo build --release &&\
  just install &&\
  just cross

