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
WORKDIR /home/rust/rustalleda
RUN cargo build --release

FROM alpine:edge
WORKDIR /home/rust/
COPY --from=builder /home/rust/target/release/rustalleda .

EXPOSE 8000

RUN apk add clang clang-libs ca-certificates

ENTRYPOINT ["./rustalleda"]
