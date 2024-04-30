build:
	docker build -f .build/Dockerfile.alpine -t wcrbrm/backup-server  .

compile:
	cargo build --release
	docker build -f .build/Dockerfile -t wcrbrm/backup-server  .

push:
	docker push wcrbrm/backup-server
