FROM paritytech/ci-linux:production AS builder

RUN apt-get update \
    && apt-get install -y git

RUN git clone --single-branch -b Phase2_Milestone1 https://github.com/WowLabz/dot-marketplace-v2.git

WORKDIR /builds/dot-marketplace-v2
RUN rustup update
RUN rustup default nightly
RUN cargo update -p parity-db
RUN cargo build --release

FROM debian:buster-slim AS runtime
WORKDIR /builds/dot-marketplace-v2
COPY --from=builder /builds/dot-marketplace-v2/target/release/node-template /usr/local/bin
EXPOSE 9944
CMD ["/usr/local/bin/node-template", "--dev", "--ws-external"]