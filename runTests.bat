cargo rustc --release --bin referee -- -C target-cpu=skylake
"./target/release/referee.exe" config REFEREE_CONFIG.json
pause