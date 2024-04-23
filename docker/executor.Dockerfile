FROM rust:latest as server_builder
WORKDIR /app

COPY . .

RUN cargo install --path thepipelinetool --bin tpt
RUN cargo install --path thepipelinetool_server --bin tpt_executor

FROM python:slim AS runner

RUN apt update \
    && DEBIAN_FRONTEND=noninteractive apt install -y \
    curl \
    && curl -fsSL https://get.docker.com -o get-docker.sh && \
    sh get-docker.sh && rm get-docker.sh \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /worker

COPY --from=server_builder /usr/local/cargo/bin/tpt /usr/local/bin/tpt
COPY --from=server_builder /usr/local/cargo/bin/tpt_executor /usr/local/bin/tpt_executor

ENV PATH="${PATH}:/usr/local/bin/"

ENTRYPOINT [ "tpt_executor" ]
