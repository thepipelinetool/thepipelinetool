FROM rust:latest as server_builder
WORKDIR /app

COPY . .

RUN cargo install --path thepipelinetool --bin tpt
RUN cargo install --path thepipelinetool_server --bin tpt_executor

FROM debian:stable-slim

WORKDIR /worker

COPY --from=server_builder /usr/local/cargo/bin/tpt /usr/local/bin/tpt
COPY --from=server_builder /usr/local/cargo/bin/tpt_executor /usr/local/bin/tpt_executor

ENV PATH="${PATH}:/usr/local/bin/"

ENTRYPOINT [ "tpt_executor" ]
