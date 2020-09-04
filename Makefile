all:
	cargo build --release
	cp target/release/rthrift_tutorial_server ./server
	cp target/release/rthrift_tutorial_client ./client

debug:
	cargo build
	cp target/debug/rthrift_tutorial_server ./server
	cp target/debug/rthrift_tutorial_client ./client
