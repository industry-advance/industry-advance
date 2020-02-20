FROM rustlang/rust:nightly
RUN apt update
RUN apt install binutils-arm-none-eabi
RUN cargo install cargo-make cargo-xbuild cargo-script
RUN rustup component add rust-src



