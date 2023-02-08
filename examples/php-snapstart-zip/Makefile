dev:
	docker-compose up dev

build:
	docker-compose up build
	sam build

deploy: build
	sam deploy

build-Php82Al2Function:
	cp -r ./app/src/main/resources/app $(ARTIFACTS_DIR)/app
	cp -r ./app/src/main/resources/nginx $(ARTIFACTS_DIR)/nginx
	cp -r ./app/src/main/resources/php $(ARTIFACTS_DIR)/php
	cp ./app/src/main/resources/run.sh $(ARTIFACTS_DIR)/run.sh
