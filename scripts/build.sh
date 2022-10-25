set -x

mkdir -p lib dist
napi build --platform --release --dts lib/index.d.ts --js lib/index.js $@
mv *.node dist

ls -al lib
ls -al dist