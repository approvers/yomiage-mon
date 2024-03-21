FROM rust:1.77.0-bullseye as builder

RUN apt-get update && \
    apt-get install -y libopus-dev && \
    rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --user-group yomiagemon
USER yomiagemon

WORKDIR /home/yomiagemon/app
COPY --chown=yomiagemon:yomiagemon . .

RUN cargo build --release --bin yomiagemon


FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y ca-certificates ffmpeg && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder --chown=root:root /home/yomiagemon/app/target/release/yomiagemon /usr/local/bin/yomiagemon

RUN useradd --create-home --user-group yomiagemon
USER yomiagemon
WORKDIR /home/yomiagemon

ENTRYPOINT [ "yomiagemon" ]