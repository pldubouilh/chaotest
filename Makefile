build::
	cargo build

run::
	PORT_CHAOS=8080 cargo run

publish:: lint
	cargo publish

lint::
	cargo clippy --all
	cargo fmt --all

doc::
	cargo doc --open