install:
	rm -rf /tmp/.out
	docker build . --tag export-code
	docker run --volume /tmp/:/tmp/ --entrypoint /bin/mv export-code /var/task/app /tmp/.out
	docker rmi -f export-code || true
	
artifacts:
	cp -r /tmp/.out $(ARTIFACTS_DIR)/app
	cp -r nginx $(ARTIFACTS_DIR)/nginx
	cp -r php $(ARTIFACTS_DIR)/php
	cp bootstrap $(ARTIFACTS_DIR)/bootstrap
	rm -rf /tmp/.out

build-Function: install artifacts
