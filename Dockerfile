FROM rust:1-slim-trixie AS build

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends just scdoc \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY justfile ./
COPY crates ./crates
COPY docs ./docs

RUN cargo build --release

RUN mkdir -p /out/bin /out/share/man/man1 \
    && for bin in chronic combine errno ifdata ifne isutf8 lckdo mispipe parallel pee sponge ts vidir vipe zrun; do \
        install -m 0755 "target/release/$bin" "/out/bin/$bin"; \
    done \
    && for src in docs/man/*.1.scd; do \
        name="$(basename "$src" .scd)"; \
        scdoc < "$src" > "/out/share/man/man1/$name"; \
        chmod 0644 "/out/share/man/man1/$name"; \
    done

FROM debian:trixie-slim

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bzip2 \
        ca-certificates \
        gzip \
        just \
        lzop \
        man-db \
        xz-utils \
        zstd \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /out /usr/local

CMD ["sh"]
