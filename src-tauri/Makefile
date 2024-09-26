
all: build

.PHONY: app build clean release

dev:
	@cargo tauri dev

build:
	@cargo tauri build

clean:
	@cargo clean

release:
	@cargo tauri build --release

app:
	xcodebuild
