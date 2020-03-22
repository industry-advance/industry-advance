FROM rustlang/rust:nightly
RUN apt update
RUN apt install -y bash binutils-arm-none-eabi xvfb xauth mgba-qt mgba-sdl
RUN cargo install cargo-make cargo-xbuild cargo-script
RUN rustup component add rust-src



