if [ "$1" = "--release" ]; then
    cargo build --release
    cp ./target/release/critical_point_gd.dll ./scene/native
    ./target/release/compiler ./scene/critical_point/resource.yml ./scene/critical_point/id.yml
else
    cargo build
    cp ./target/debug/critical_point_gd.dll ./scene/native
    ./target/debug/compiler ./scene/critical_point/resource.yml ./scene/critical_point/id.yml
fi
