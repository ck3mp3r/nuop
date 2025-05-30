IMAGE_NAME=nuop
VERSION=latest
REGISTRY=ghcr.io/ck3mp3r

tests:
	cd operator && cargo test

build:
	docker build --debug -f operator/docker/Dockerfile . -t $(REGISTRY)/$(IMAGE_NAME):$(VERSION) 
	kind load docker-image $(REGISTRY)/$(IMAGE_NAME):latest -n nuop 
	
buildx:
	docker buildx create --name mybuilder --use || true
	docker buildx inspect --bootstrap
	docker buildx build -f ./operator/docker/Dockerfile --platform linux/amd64,linux/arm64 -t $(REGISTRY)/$(IMAGE_NAME):$(VERSION) --push ./operator

clean:
	cd operator && cargo clean

clippy:
	cd operator && cargo clippy

fmt:
	cd operator && cargo fmt

crds:
	cd operator && cargo run --bin generate > chart/crds/nuop.yaml

coverage:
	cd operator && cargo tarpaulin --out Html
	
act-test:
	@act push \
		--rm \
		--container-architecture linux/amd64 \
		-s GITHUB_TOKEN=${GITHUB_TOKEN} \
		-s ACTIONS_RUNTIME_TOKEN=${GITHUB_TOKEN} \
		-P ubuntu-latest=catthehacker/ubuntu:js-latest \
		-W .github/workflows/test.yaml \
		-j test
