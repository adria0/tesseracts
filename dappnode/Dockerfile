FROM alpine:edge AS builder

RUN apk add build-base \
    cmake \
    linux-headers \
    openssl-dev \
    cargo \
    clang \
    clang-libs \
    git

WORKDIR /home/rust/
COPY . .
WORKDIR /home/rust/tesseracts
RUN cargo build --release

FROM alpine:edge
WORKDIR /home/rust/
COPY --from=builder /home/rust/target/release/tesseracts .
COPY --from=builder /home/rust/mainnet.toml .

EXPOSE 80

RUN apk add clang clang-libs ca-certificates

ENTRYPOINT ["./tesseracts","--cfg","mainnet.toml","-vvv"]