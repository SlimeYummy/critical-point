if [ "$1" = "--release" ]; then
    cargo build --release
    cp ./target/release/critical_point_gd.dll ./scene
else
    cargo build
    cp ./target/debug/critical_point_gd.dll ./scene
fi
