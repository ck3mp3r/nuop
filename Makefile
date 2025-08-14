IMAGE_NAME=nuop
VERSION=latest
REGISTRY=ghcr.io/ck3mp3r

tests:
	cd operator && cargo test

build:
	cd operator && docker build --debug -f docker/Dockerfile . -t $(REGISTRY)/$(IMAGE_NAME):$(VERSION) 
	kind load docker-image $(REGISTRY)/$(IMAGE_NAME):latest -n nuop 
	
buildx:
	cd operator && docker buildx create --name mybuilder --use || true
	cd operator && docker buildx inspect --bootstrap
	cd operator && docker buildx build -f docker/Dockerfile --platform linux/amd64,linux/arm64 -t $(REGISTRY)/$(IMAGE_NAME):$(VERSION) --push .

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
		--container-options "--network bridge --dns 8.8.8.8 --dns 1.1.1.1" \
		--container-architecture linux/aarch64 \
		-s GITHUB_TOKEN=${GITHUB_TOKEN} \
		-s ACTIONS_RUNTIME_TOKEN=${GITHUB_TOKEN} \
		-P ubuntu-latest=catthehacker/ubuntu:js-latest \
		-W .github/workflows/test.yaml \
		-j test

act-buildx:
	@act workflow-dispatch \
		--rm \
		--container-architecture linux/aarch64 \
		--privileged \
		--container-daemon-socket /var/run/docker.sock \
		-s GITHUB_TOKEN=${GITHUB_TOKEN} \
		-s ACTIONS_RUNTIME_TOKEN=${GITHUB_TOKEN} \
		-P ubuntu-latest=catthehacker/ubuntu:js-latest \
		-W .github/workflows/buildx.yaml \
		-j build
