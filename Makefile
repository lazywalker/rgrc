PREFIX=/usr/local

all:
	cargo build

clean:
	cargo clean

release-build: rgrc.1.gz
	cargo build --release

release: rgrc.1.gz
	cargo build --release

test:
	cargo test

man:
# 	pandoc -f markdown -t man doc/rgrc.1.md -o doc/rgrc.1
	script/md2man.sh doc/rgrc.1.md doc/rgrc.1

install: release
	install -D -m 0755 target/release/rgrc $(PREFIX)/bin/rgrc
	install -D -m 0644 doc/rgrc.1.gz $(PREFIX)/share/man/man1/
	install -D -m 0644 etc/rgrc.* $(PREFIX)/etc/
	mkdir -p $(PREFIX)/share/rgrc
	install -D -m 0644 share/conf.* $(PREFIX)/share/rgrc/

uninstall:
	rm -f $(PREFIX)/bin/rgrc
	rm -f $(PREFIX)/share/man/man1/rgrc.1.gz
	rm -f $(PREFIX)/etc/rgrc.*
	rm -rf $(PREFIX)/share/rgrc

 rgrc.1.gz:  doc/rgrc.1
	gzip -fk doc/rgrc.1
