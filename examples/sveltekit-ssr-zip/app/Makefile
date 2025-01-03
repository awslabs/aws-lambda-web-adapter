BUILD_DIR=${CURDIR}/build

# delete:
# 1. ./build directory created by @sveltejs/adapter-node
# 2. app ./node_modules
clean:
	rm -rf ./build
	rm -rf ./node_modules

install:
	npm install

# build @sveltejs/adapter-node app with prod deps: https://svelte.dev/docs/kit/adapter-node#Deploying
build:
	npm run build
	cp ./run.sh $(BUILD_DIR)
	cp ./package*.json $(BUILD_DIR)
	cd $(BUILD_DIR) && npm ci --omit dev

build-SvelteKitSsrFunction: clean install build
	cp -r $(BUILD_DIR)/. $(ARTIFACTS_DIR)