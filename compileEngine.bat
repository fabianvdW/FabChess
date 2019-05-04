cargo rustc --release --bin schach_reworked -- -C target-cpu=native
cd target
cd release
schach_reworked.exe
pause
