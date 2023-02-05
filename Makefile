.PHONY: all build-homie build-frontend copy-to-server

PROFILE ?= "dev"
TARGET_ARG := $(if $(TARGET),--target=$(TARGET),)
SERVER_DIRECTORY ?= "~/opt/homie"

all: build-homie

build-homie: build-frontend
	cargo build --profile ${PROFILE} ${TARGET_ARG}

build-frontend:
	cd frontend && pnpm install
	cd frontend && pnpm build

copy-to-server:
	ssh ${SERVER_ADDRESS} 'mkdir -p ${SERVER_DIRECTORY}'
	rsync ./target/release/homie ./target/release/db ${SERVER_ADDRESS}:${SERVER_DIRECTORY}
