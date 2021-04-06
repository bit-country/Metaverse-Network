# Based from https://github.com/paritytech/substrate/blob/master/.maintain/Dockerfile

FROM phusion/baseimage:0.10.2 as builder
LABEL maintainer="justin@bit.country"
LABEL description="This is the build stage for Bit.Country Network Node. Here we create the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
ARG GIT_COMMIT=
ENV GIT_COMMIT=$GIT_COMMIT
WORKDIR /bitCountry

COPY . /bitCountry

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake cmake pkg-config libssl-dev git clang libclang-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup default nightly-2021-03-04 && \
	rustup target add wasm32-unknown-unknown --toolchain nightly-2021-03-04 && \
	cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer="justin@bit.country"
LABEL description="This is the 2nd stage: a very small image where we copy the Bit.Country Node binary."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	useradd -m -u 1000 -U -s /bin/sh -d /bitCountry bitCountry && \
	mkdir -p /bitCountry/.local/share/bitCountry && \
	chown -R bitCountry:bitCountry /bitCountry/.local && \
	ln -s /bitCountry/.local/share/bitCountry /data

# Add the binary file generated in the `builder` container above
COPY --from=builder /bitCountry/target/$PROFILE/bitcountry-parachain /usr/local/bin

# checks
RUN ldd /usr/local/bin/bitcountry-parachain && \
	/usr/local/bin/bitcountry-parachain --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

USER bitCountry
EXPOSE 30333 9933 9944 9615

RUN mkdir /bitCountry/data

VOLUME ["/bitCountry/data"]

CMD ["/usr/local/bin/bitcountry-parachain"]