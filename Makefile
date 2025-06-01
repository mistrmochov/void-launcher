APP_NAME = void-launcher
BUILD_DIR = target/release
BIN = $(BUILD_DIR)/$(APP_NAME)
INSTALL_BIN = /usr/bin
INSTALL_SHARE = /usr/share/$(APP_NAME)

SRC = $(shell find src -name '*.rs')
RESOURCES = \
	src/resources/config.json \
	src/resources/style.css \
	src/resources/back.css \
	src/resources/dark.css \
	src/resources/light.css \
	src/resources/void.ui

all: $(BIN)

$(BIN): $(SRC) $(RESOURCES) Cargo.toml Cargo.lock
	cargo build --release

install: all
	@echo "Installing binary..."
	install -Dm755 $(BIN) $(INSTALL_BIN)/$(APP_NAME)

	@echo "Installing resources..."
	mkdir -p $(INSTALL_SHARE)/icons
	cp -r src/resources/drawable/* $(INSTALL_SHARE)/icons/

uninstall:
	@echo "Uninstalling..."
	rm -f $(INSTALL_BIN)/$(APP_NAME)
	rm -rf $(INSTALL_SHARE)

clean:
	cargo clean

.PHONY: all install uninstall clean
