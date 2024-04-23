FROM rust:latest as server_builder
WORKDIR /app

COPY . .

RUN cargo install --path thepipelinetool --bin tpt
RUN cargo install --path thepipelinetool_server --bin server

FROM ghcr.io/cirruslabs/flutter:stable as web_builder
WORKDIR /app

COPY thepipelinetool_ui .
WORKDIR /app/thepipelinetool_ui

RUN flutter pub get
RUN flutter build web --release

FROM debian:stable-slim
WORKDIR /server

COPY --from=web_builder /app/build/web static/
COPY --from=server_builder /usr/local/cargo/bin/tpt /usr/local/bin/tpt
COPY --from=server_builder /usr/local/cargo/bin/server /usr/local/bin/server

ENV PATH="${PATH}:/usr/local/bin/"

EXPOSE 8000

CMD server
