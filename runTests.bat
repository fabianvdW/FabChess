cargo rustc --release --bin spielleiter -- -C target-cpu=skylake
cd target
cd release
spielleiter.exe
pause