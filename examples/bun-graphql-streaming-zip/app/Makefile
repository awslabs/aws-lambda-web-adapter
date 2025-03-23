.PHONY: build-BunGraphqlFunction build

build-BunGraphqlFunction:
				bun install
				bun run build
				mkdir -p $(ARTIFACTS_DIR)
				cp run.sh out/
				cp -r out/* $(ARTIFACTS_DIR)/

build:
				$(MAKE) build-BunGraphqlFunction
