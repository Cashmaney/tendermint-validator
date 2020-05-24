# Simple usage with a mounted data directory:
# > docker build -t enigma .
# > docker run -it -p 26657:26657 -p 26656:26656 -v ~/.enigmad:/root/.enigmad -v ~/.enigmacli:/root/.enigmacli enigma enigmad init
# > docker run -it -p 26657:26657 -p 26656:26656 -v ~/.enigmad:/root/.enigmad -v ~/.enigmacli:/root/.enigmacli enigma enigmad start
FROM baiduxlab/sgx-rust:1804-1.1.2 AS build-env-rust-go

ENV PATH="/root/.cargo/bin:$PATH"
ENV GOROOT=/usr/local/go
ENV GOPATH=/go/
ENV PATH=$PATH:/usr/local/go/bin:$GOPATH/bin


RUN curl -O https://dl.google.com/go/go1.14.2.linux-amd64.tar.gz
RUN tar -C /usr/local -xzf go1.14.2.linux-amd64.tar.gz
# Set working directory for the build

WORKDIR /go/src/tendermint-validator/

ARG SGX_MODE=HW
ENV SGX_MODE=${SGX_MODE}
ENV MITIGATION_CVE_2020_0551=LOAD

COPY tee_validator/third_party/build tee_validator/third_party/build

# Add source files
COPY tee_validator/ tee_validator/

COPY Makefile Makefile

RUN make clean
RUN make vendor

WORKDIR /go/src/tendermint-validator/

RUN . /opt/sgxsdk/environment && env && MITIGATION_CVE_2020_0551=LOAD SGX_MODE=${SGX_MODE} make build-rust

# Add source files
COPY cmd cmd
COPY internal internal
COPY go.mod .
COPY go.sum .

RUN . /opt/sgxsdk/environment && env && MITIGATION_CVE_2020_0551=LOAD SGX_MODE=${SGX_MODE} make build/signer

# Final image
FROM cashmaney/enigma-sgx-base

ENV ENCLAVE_DIR=/usr/lib/

# Install ca-certificates
WORKDIR /root

# Copy over binaries from the build-env
COPY --from=build-env-rust-go /go/src/tendermint-validator/build/enclave.signed.so /usr/lib/
COPY --from=build-env-rust-go /go/src/tendermint-validator/tee_validator/go-bridge/api/libgo_bridge.so /usr/lib/
COPY --from=build-env-rust-go /go/src/tendermint-validator/build/signer /usr/bin/signer

RUN mkdir -p /root/.signer/

COPY ./packaging_docker/tendermint-validator/config.toml /root/.signer/config/config.toml
COPY ./packaging_docker/tendermint-validator/priv_validator_state.json /root/.signer/watermark/enigma-1_priv_validator_state.json
COPY ./packaging_docker/signer_init.sh /root/

RUN chmod +x signer_init.sh

# don't feel like debugging paths right now
# COPY /opt/sgxsdk/lib64/libsgx_uae_service.so /usr/lib/libsgx_uae_service.so

# Run enigmad by default, omit entrypoint to ease using container with enigmacli
ENTRYPOINT ["/bin/bash", "signer_init.sh"]