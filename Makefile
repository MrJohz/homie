.PHONY: all build-homie build-frontend copy-to-server

PROFILE ?= "dev"
TARGET_ARG := $(if $(TARGET),--target=$(TARGET),)
TARGET_DIRECTORY := $(if $(TARGET),target/$(TARGET),target)
ifeq ($(PROFILE),dev)
  PROFILE_DIRECTORY=debug
else
  PROFILE_DIRECTORY=$(PROFILE)
endif
SERVER_DIRECTORY ?= "~/opt/homie"

all: build-homie

build-homie: build-frontend
	cross build --profile ${PROFILE} ${TARGET_ARG}

build-frontend:
	cd frontend && pnpm install
	cd frontend && pnpm build

copy-to-server: build-homie
	rsync --mkpath ./${TARGET_DIRECTORY}/${PROFILE_DIRECTORY}/homie ./${TARGET_DIRECTORY}/${PROFILE_DIRECTORY}/db ${SERVER_ADDRESS}:${SERVER_DIRECTORY}
