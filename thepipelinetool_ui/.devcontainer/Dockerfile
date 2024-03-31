FROM ubuntu:jammy

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
	apt-get install -y sudo wget curl unzip clang cmake git ninja-build pkg-config libgtk-3-dev liblzma-dev libstdc++-12-dev && \
    git clone https://github.com/flutter/flutter.git -b stable --depth 1

ENV PATH "$PATH:/flutter/bin"

RUN flutter config --no-analytics --enable-web && \
    flutter precache && \
	flutter doctor