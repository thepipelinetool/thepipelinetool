FROM rust:latest as server_builder
WORKDIR /app

COPY . .

RUN cargo install --path thepipelinetool --bin tpt
RUN cargo install --path thepipelinetool_server --bin worker
RUN cargo install --path thepipelinetool_server --bin tpt_executor

FROM python:slim AS runner

COPY --from=docker:dind /usr/local/bin/docker /usr/local/bin/

WORKDIR /worker

COPY --from=server_builder /usr/local/cargo/bin/tpt /usr/local/bin/tpt
COPY --from=server_builder /usr/local/cargo/bin/worker /usr/local/bin/worker
COPY --from=server_builder /usr/local/cargo/bin/tpt_executor /usr/local/bin/tpt_executor

ENV PATH="${PATH}:/usr/local/bin/"

CMD [ "worker" ]
