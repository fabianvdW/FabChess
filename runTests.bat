cargo rustc --release --bin referee -- -C target-cpu=skylake
cd target
cd release
spielleiter.exe
pause