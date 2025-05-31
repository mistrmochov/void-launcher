APP_NAME = void-launcher
BUILD_DIR = target/release
INSTALL_BIN = /usr/bin
INSTALL_SHARE = /usr/share/$(APP_NAME)
BUILD_BIN = $(BUILD_DIR)/$(APP_NAME)

all: $(BUILD_BIN)

$(BUILD_BIN):
	cargo build --release

install: all
	@echo "Installing binary..."
	install -Dm755 $(BUILD_BIN) $(INSTALL_BIN)/$(APP_NAME)

	@echo "Installing resources..."
	mkdir -p $(INSTALL_SHARE)/icons
	cp -r src/resources/drawable/* $(INSTALL_SHARE)/icons/

uninstall:
	@echo "Uninstalling..."
	rm -f $(INSTALL_BIN)/$(APP_NAME)
	rm -rf $(INSTALL_SHARE)

clean:
	@echo "Cleaning build artifacts..."
	cargo clean

.PHONY: all install uninstall clean
