VERSION = 0.0.1
# GOLINT:=$(shell go list -f {{.Target}} golang.org/x/lint/golint)
UNAME_S = $(shell uname -s)

ENCLAVE_NAME=enclave.signed.so

all: build

build: build-rust build/signer

build-rust:
	cd tee_validator && make && make client
	cp tee_validator/bin/$(ENCLAVE_NAME) .

build/signer: cmd/signer/main.go $(wildcard internal/**/*.go tee_validator/**/*.go)
	CGO_ENABLED=1 go build -o ./build/signer ${gobuild_flags} ./cmd/signer
	cp $(ENCLAVE_NAME) ./build/

vendor:
	cargo vendor tee_validator/third_party/vendor --manifest-path tee_validator/third_party/build/Cargo.toml

deb: build
    ifneq ($(UNAME_S),Linux)
		exit 1
    endif
	rm -rf /tmp/TendermintValidator

	mkdir -p /tmp/TendermintValidator/deb/bin
	mv -f ./build/signer /tmp/TendermintValidator/deb/bin/signer
	chmod +x /tmp/TendermintValidator/deb/bin/signer

	mkdir -p /tmp/TendermintValidator/deb/DEBIAN/
	mkdir -p /tmp/TendermintValidator/deb/opt/tendermint-validator/

	cp -r ./packaging_ubuntu/opt/* /tmp/TendermintValidator/deb/opt/

	cp ./packaging_ubuntu/control /tmp/TendermintValidator/deb/DEBIAN/control
	echo "" >> /tmp/TendermintValidator/deb/DEBIAN/control
	cp ./packaging_ubuntu/postinst /tmp/TendermintValidator/deb/DEBIAN/postinst
	chmod 755 /tmp/TendermintValidator/deb/DEBIAN/postinst
	cp ./packaging_ubuntu/postrm /tmp/TendermintValidator/deb/DEBIAN/postrm
	chmod 755 /tmp/TendermintValidator/deb/DEBIAN/postrm
	dpkg-deb --build /tmp/TendermintValidator/deb/ .
	-rm -rf /tmp/TendermintValidator


#lint: tools
#	@$(GOLINT) -set_exit_status ./...

test:
	@go test -short ./...

race:
	@go test -race -short ./...

msan:
	@go test -msan -short ./...

#tools:
#	@go install golang.org/x/lint/golint

clean:
	rm -rf build
	cd tee_validator &&	make clean

.PHONY: all lint test race msan tools clean build
