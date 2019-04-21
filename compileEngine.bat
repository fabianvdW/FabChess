cargo rustc --release --bin schach_reworked -- -C target-cpu=skylake
cd target
cd release
schach_reworked.exe
pause
