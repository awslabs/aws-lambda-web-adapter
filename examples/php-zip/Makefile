install-%:
	rm -rf .out
	PLATFORM=${*} docker-compose build export-code
	PLATFORM=${*} docker-compose up export-code
	PLATFORM=${*} docker-compose down

artifacts:
	cp -r .out $(ARTIFACTS_DIR)/app
	cp -r nginx $(ARTIFACTS_DIR)/nginx
	cp -r php $(ARTIFACTS_DIR)/php
	cp bootstrap $(ARTIFACTS_DIR)/bootstrap

build-Function: install-amd64 artifacts
build-FunctionArm64: install-arm64 artifacts
