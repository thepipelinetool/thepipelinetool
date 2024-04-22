FROM rust:latest as server_builder
WORKDIR /app

COPY . .

RUN cargo install --path thepipelinetool --bin tpt
RUN cargo install --path thepipelinetool_server --bin worker
RUN cargo install --path thepipelinetool_server --bin tpt_executor

FROM ubuntu:latest AS runner

RUN apt update \
    && DEBIAN_FRONTEND=noninteractive apt install -y \
    curl

RUN curl -fsSL https://get.docker.com -o get-docker.sh && \
    sh get-docker.sh && rm get-docker.sh
WORKDIR /worker

COPY --from=server_builder /usr/local/cargo/bin/tpt /usr/local/bin/tpt
COPY --from=server_builder /usr/local/cargo/bin/worker /usr/local/bin/worker
COPY --from=server_builder /usr/local/cargo/bin/tpt_executor /usr/local/bin/tpt_executor

# RUN sh -c 'apk add --no-cache bash'

ENV PATH="${PATH}:/usr/local/bin/"

# ENTRYPOINT [ "worker" ]
CMD [ "worker" ]
