FROM rust:latest as server_builder
WORKDIR /app

COPY . .

RUN cargo install --path thepipelinetool_cli --bin tpt
RUN cargo install --path thepipelinetool_server --bin worker

FROM rust:latest
WORKDIR /worker

COPY --from=server_builder /usr/local/cargo/bin/tpt /usr/local/bin/tpt
COPY --from=server_builder /usr/local/cargo/bin/worker /usr/local/bin/worker

CMD worker
