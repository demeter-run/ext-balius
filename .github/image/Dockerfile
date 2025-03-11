FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
ARG BIN
COPY bin/${BIN}-Linux-${TARGETARCH} /bin/${BIN}
RUN chmod +x /bin/${BIN}
RUN ln -s /bin/${BIN} /${BIN}

ENTRYPOINT [ "${BIN}" ]
