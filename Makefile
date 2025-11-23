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
	cargo build --release

macos:
	cargo build --release --target x86_64-apple-darwin

armv7:
	cargo build --release --target armv7-unknown-linux-musleabihf

linux:
	cargo build --release --target x86_64-unknown-linux-musl

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
	cargo clippy --all

test:
	cargo test

fmt:
	cargo fmt --all

man:
# 	pandoc -f markdown -t man doc/rgrc.1.md -o doc/rgrc.1
	script/md2man.sh doc/rgrc.1.md doc/rgrc.1

install: release
	install -D -m 0755 target/release/rgrc $(PREFIX)/bin/rgrc
	install -D -m 0644 doc/rgrc.1.gz $(PREFIX)/share/man/man1/
	install -D -m 0644 etc/rgrc.* $(PREFIX)/etc/
	install -d $(PREFIX)/share/rgrc
	install -D -m 0644 share/conf.* $(PREFIX)/share/rgrc/

uninstall:
	rm -f $(PREFIX)/bin/rgrc
	rm -f $(PREFIX)/share/man/man1/rgrc.1.gz
	rm -f $(PREFIX)/etc/rgrc.*
	rm -rf $(PREFIX)/share/rgrc

 rgrc.1.gz:  doc/rgrc.1
	gzip -fk doc/rgrc.1
