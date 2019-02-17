npm i -g @dappnode/dappnodesdk
rm -rf build
mkdir build
rsync -av --progress ../ build --exclude .git --exclude dappnode --exclude data --exclude target
cp mainnet.toml Dockerfile build
dappnodesdk build
dappnodesdk build