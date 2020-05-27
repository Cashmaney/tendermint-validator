package api

// #cgo LDFLAGS: -Wl,-rpath,${SRCDIR} -L${SRCDIR} -lgo_bridge
// #include <stdlib.h>
// #include "bindings.h"
import "C"
import (
	"fmt"
)

// nice aliases to the rust names
type i32 = C.int32_t
type i64 = C.int64_t
type u64 = C.uint64_t
type u32 = C.uint32_t
type u8 = C.uint8_t
type u8_ptr = *C.uint8_t
type usize = C.uintptr_t
type cint = C.int

func CheckEnclave() error {
	errmsg := C.Buffer{}
	_, err := C.health_check(&errmsg)
	if err != nil {
		return errorWithMessage(err, errmsg)
	}
	return nil
}

// KeyGen Seng KeyGen request to enclave
func GetPubKey() ([]byte, error) {
	errmsg := C.Buffer{}
	res, err := C.get_pubkey(&errmsg)
	if err != nil {
		return nil, errorWithMessage(err, errmsg)
	}
	return receiveSlice(res), nil
}

// KeyGen Seng KeyGen request to enclave
func Sign(bytes []byte) ([]byte, error) {
	slice := sendSlice(bytes)
	errmsg := C.Buffer{}
	res, err := C.sign(slice, &errmsg)
	if err != nil {
		return nil, errorWithMessage(err, errmsg)
	}
	return receiveSlice(res), nil
}

// Import a private key into the enclave
func Import(bytes []byte, password []byte) error {
	slice := sendSlice(bytes)
	passwordSlice := sendSlice(password)
	errmsg := C.Buffer{}
	_, err := C.import_key(slice, passwordSlice, &errmsg)
	if err != nil {
		return errorWithMessage(err, errmsg)
	}
	return nil
}

// Generate new key inside the enclave
func Generate(password []byte) error {
	passwordSlice := sendSlice(password)
	errmsg := C.Buffer{}
	_, err := C.generate_key(passwordSlice, &errmsg)
	if err != nil {
		return errorWithMessage(err, errmsg)
	}
	return nil
}

//// Export a public key from the enclave
func Export(password []byte) ([]byte, error) {
	slice := sendSlice(password)
	errmsg := C.Buffer{}
	res, err := C.export_key(slice, &errmsg)
	if err != nil {
		return nil, errorWithMessage(err, errmsg)
	}
	return receiveSlice(res), nil
}

/**** To error module ***/

func errorWithMessage(err error, b C.Buffer) error {
	msg := receiveSlice(b)
	if msg == nil {
		return err
	}
	return fmt.Errorf("%s", string(msg))
}
