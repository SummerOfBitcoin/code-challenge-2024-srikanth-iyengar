.PHONY: mine

mine:
	@ cargo run --release
test:
	@ cargo t -- --nocapture
