FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY bin/controller-Linux-${TARGETARCH} /bin/controller
RUN chmod +x /bin/controller
RUN ln -s /bin/controller /controller

ENTRYPOINT [ "controller" ]
