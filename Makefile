all:
	cargo build --release --features=hmetis --target=x86_64-apple-darwin
	install_name_tool -add_rpath /usr/local/lib target/x86_64-apple-darwin/release/metapartition