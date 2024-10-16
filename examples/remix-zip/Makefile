build-RemixFunction:
	cd remix-app && npm install && npm run build && npm prune --omit=dev
	cd remix-app && cp -R build node_modules server.js package.json ../run.sh $(ARTIFACTS_DIR)