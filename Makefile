PREFIX=/usr/local
VER=$(shell cargo pkgid | cut -d\# -f2 | cut -d: -f2)
APP_VERSION=${VER}
TARBALL="target/tarball"
APP_NAME="rgrc"
	
all:
	cargo build

clean:
	cargo clean

ver:
	@echo Version: ${APP_NAME} v${APP_VERSION}

release: rgrc.1.gz
	cargo auditable build --release
	
minimal: rgrc.1.gz
	cargo auditable build --profile minimal

macos:
	cargo auditable build --release --target x86_64-apple-darwin

armv7:
	cargo auditable build --release --target armv7-unknown-linux-musleabihf

linux:
	cargo auditable build --release --target x86_64-unknown-linux-musl

bin: macos linux armv7
	@echo Creating tarball...
	@mkdir -p ${TARBALL}
	
	@echo Creating x86_64-apple-darwin
	@tar cvfz "${TARBALL}/${APP_NAME}-${APP_VERSION}-x86_64-apple-darwin.tar.gz" -C target/x86_64-apple-darwin/release/ ${APP_NAME} 

	@echo Creating x86_64-unknown-linux-musl
	@tar cvfz "${TARBALL}/${APP_NAME}-${APP_VERSION}-x86_64-unknown-linux-musl.tar.gz" -C target/x86_64-unknown-linux-musl/release/ ${APP_NAME}

	@echo Creating armv7-unknown-linux-musleabihf
	@tar cvfz "${TARBALL}/${APP_NAME}-${APP_VERSION}-armv7-unknown-linux-musleabihf.tar.gz" -C target/armv7-unknown-linux-musleabihf/release/ ${APP_NAME}

data: rgrc.1.gz
	@echo Creating data tarball...
	@mkdir -p ${TARBALL}
	@tar cvfz "${TARBALL}/${APP_NAME}-data-${APP_VERSION}.tar.gz" doc/*.gz etc/ share/

lint:
	cargo clippy --all-features

test:
	cargo test --all-features

fmt:
	cargo fmt --all

check: lint fmt
	@echo "\033[33mcargo lint and fmt done\033[0m"

man:
	pandoc --standalone -f markdown -t man doc/rgrc.1.md -o doc/rgrc.1
# 	script/md2man.sh doc/rgrc.1.md doc/rgrc.1

install: release
	install -Dm 0755 target/release/${APP_NAME} -t $(PREFIX)/bin/
	install -Dm 0644 doc/${APP_NAME}.1.gz -t $(PREFIX)/share/man/man1/
	install -Dm 0644 etc/${APP_NAME}.* -t $(PREFIX)/etc/
	install -Dm 0644 share/conf.* -t $(PREFIX)/share/${APP_NAME}/

uninstall:
	rm -f $(PREFIX)/bin/rgrc
	rm -f $(PREFIX)/share/man/man1/rgrc.1.gz
	rm -f $(PREFIX)/etc/rgrc.*
	rm -rf $(PREFIX)/share/rgrc

 rgrc.1.gz:  doc/rgrc.1
	gzip -fk doc/rgrc.1

deb: rgrc.1.gz
# 	cargo deb --no-default-features
	cargo clean
	docker run --rm -v "$(pwd):/work" rgrc-deb-builder cargo deb --no-default-features