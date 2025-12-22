FROM rust:alpine

WORKDIR /app

RUN apk add musl-dev

COPY . .

RUN cargo build --release

RUN strip target/release/cultbot

FROM scratch

WORKDIR /app

COPY --from=0 /app/target/release/cultbot /app

ENTRYPOINT ["./cultbot"]
