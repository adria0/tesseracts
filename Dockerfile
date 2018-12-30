FROM buildpack-deps:stretch AS build

RUN curl https://sh.rustup.rs -sSf | sh -vs -- -y 
RUN $HOME/.cargo/bin/rustup install nightly 
RUN $HOME/.cargo/bin/rustup default nightly

WORKDIR /home/rust/
COPY . .
RUN $HOME/.cargo/bin/cargo build --release

FROM scratch
WORKDIR /home/rust/
COPY --from=build /home/rust/target/release/rust-mbe .

EXPOSE 8000

ENTRYPOINT ["./rust-mbe"]
