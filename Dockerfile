FROM rust:alpine

WORKDIR /app

COPY . .

RUN apk add musl-dev \
    && cargo build --release

FROM scratch

WORKDIR /app

COPY --from=0 /app/target/release/cultbot /app

ENTRYPOINT ["./cultbot"]
