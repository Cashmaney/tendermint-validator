VERSION = 0.0.1
GOLINT:=$(shell go list -f {{.Target}} golang.org/x/lint/golint)
UNAME_S = $(shell uname -s)
all: build

build: build/signer

build/signer: cmd/signer/main.go $(wildcard internal/**/*.go)
	CGO_ENABLED=0 go build -o ./build/signer ${gobuild_flags} ./cmd/signer

deb: build
    ifneq ($(UNAME_S),Linux)
		exit 1
    endif
	rm -rf /tmp/TendermintValidator

	mkdir -p /tmp/TendermintValidator/deb/bin
	mv -f ./build/signer /tmp/TendermintValidator/deb/bin/signer
	chmod +x /tmp/TendermintValidator/deb/bin/signer

	mkdir -p /tmp/TendermintValidator/deb/DEBIAN
	cp ./packaging_ubuntu/control /tmp/TendermintValidator/deb/DEBIAN/control
	echo "" >> /tmp/TendermintValidator/deb/DEBIAN/control
	cp ./packaging_ubuntu/postinst /tmp/TendermintValidator/deb/DEBIAN/postinst
	chmod 755 /tmp/TendermintValidator/deb/DEBIAN/postinst
	cp ./packaging_ubuntu/postrm /tmp/TendermintValidator/deb/DEBIAN/postrm
	chmod 755 /tmp/TendermintValidator/deb/DEBIAN/postrm
	dpkg-deb --build /tmp/TendermintValidator/deb/ .
	-rm -rf /tmp/TendermintValidator


lint: tools
	@$(GOLINT) -set_exit_status ./...

test:
	@go test -short ./...

race:
	@go test -race -short ./...

msan:
	@go test -msan -short ./...

tools:
	@go install golang.org/x/lint/golint

clean:
	rm -rf build

.PHONY: all lint test race msan tools clean build
