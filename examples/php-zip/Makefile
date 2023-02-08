install%:
	rm -rf .out
	docker-compose build export-code
	docker-compose up export-code
	docker-compose down

artifacts:
	cp -r .out $(ARTIFACTS_DIR)/app
	cp -r nginx $(ARTIFACTS_DIR)/nginx
	cp -r php $(ARTIFACTS_DIR)/php
	cp bootstrap $(ARTIFACTS_DIR)/bootstrap

build-Function: install artifacts
