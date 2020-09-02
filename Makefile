all:
	cargo build
	cp target/debug/rthrift_tutorial_server ./server
	cp target/debug/rthrift_tutorial_client ./client

run_server:
	./server

run_client:
	./client

