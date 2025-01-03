BUILD_DIR=${CURDIR}/build

clean:
	rm -rf ./node_modules
	rm -rf ./build

install:
	npm install

build:
	npm run build
	cp ./package.json $(BUILD_DIR)
	cp ./run.sh $(BUILD_DIR)

build-SvelteKitSsrFunction: clean install build
	cp -r $(BUILD_DIR)/. $(ARTIFACTS_DIR)