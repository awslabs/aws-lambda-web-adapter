build-HelloWorldFunction:
	cp *.go $(ARTIFACTS_DIR)
	cp go.mod go.sum $(ARTIFACTS_DIR)
	cp run.sh $(ARTIFACTS_DIR)
	env GOOS=linux go build -ldflags="-s -w" -o $(ARTIFACTS_DIR)/bin/handler $(ARTIFACTS_DIR)/*.go
