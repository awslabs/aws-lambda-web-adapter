install:
	npm install

build:
	npm run build

artifacts:
	# Static resources ideally served from CloudFront
	cp -r public/ .next/standalone/public
	cp -r .next/static/ .next/standalone/.next/static
	
	# Copy artifacts for deployment
	cp -r .next/standalone/ $(ARTIFACTS_DIR)
	cp run.sh $(ARTIFACTS_DIR)

build-NextjsFunction: install build artifacts