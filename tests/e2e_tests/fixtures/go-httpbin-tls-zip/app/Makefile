build-HttpBinFunction:
	GOOS=linux GOARCH=amd64 go install github.com/mccutchen/go-httpbin/v2/cmd/go-httpbin@v2.4.1
	mv ~/go/bin/linux_amd64/go-httpbin $(ARTIFACTS_DIR)/
	cp bootstrap $(ARTIFACTS_DIR)/
	cp -R certs $(ARTIFACTS_DIR)/