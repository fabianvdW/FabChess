EXE     = FabChess

rule:
	cargo rustc --release -p uci-engine -- -C target-cpu=native --emit link=$(EXE)