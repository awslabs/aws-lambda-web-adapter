build-HttpBinFunction:
	GOBIN=$(ARTIFACTS_DIR)/ go install github.com/mccutchen/go-httpbin/v2/cmd/go-httpbin@v2.4.1
	cp bootstrap $(ARTIFACTS_DIR)/
	cp -R certs $(ARTIFACTS_DIR)/