.PHONY: build-GraphqlYogaFunction build

build-GraphqlYogaFunction:
				pnpm i
				pnpm build
				mkdir -p $(ARTIFACTS_DIR)
				cp -r dist/* $(ARTIFACTS_DIR)/

build:
				$(MAKE) build-GraphqlYogaFunction
