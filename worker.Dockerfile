FROM rust:latest as builder

WORKDIR /app
COPY thepipelinetool thepipelinetool
COPY runner runner
COPY task task
COPY utils utils
WORKDIR /app/thepipelinetool
RUN cargo install --path . --examples --root .

FROM rust:latest as server_builder

WORKDIR /app
COPY thepipelinetool thepipelinetool
COPY runner runner
COPY task task
COPY utils utils
WORKDIR /app/server
COPY server/src/dummy.rs .
COPY server/Cargo.toml .
RUN sed -i 's#bin/worker.rs#dummy.rs#' Cargo.toml
RUN cargo install --path . --bin worker

COPY server/Cargo.toml .
COPY server/src src
COPY server/bin bin
RUN cargo install --path . --bin worker

FROM rust:latest
WORKDIR /worker
COPY --from=server_builder /usr/local/cargo/bin/worker /usr/local/bin/worker
COPY --from=builder /app/thepipelinetool/bin/ /worker/bin/

EXPOSE 8000

CMD worker
