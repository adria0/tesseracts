GETH_DOCKER_PARAMS="-v `pwd`/config:/config -v `pwd`/data:/data -p 8545:8545  -t ethereum/client-go:alltools-v1.8.27"

if [ ! "$(docker ps -q -f name=geth)" ]; then
    echo [geth] Container does not exist
    if [ "$(docker ps -aq -f status=exited -f name=geth)" ]; then
        echo [geth] Found old one, cleaning up
	docker rm geth
    fi
    
    echo [geth] Starting new container...
    docker run $GETH_DOCKER_PARAMS geth --datadir data init config/genesis.json
    docker run -d --name geth $GETH_DOCKER_PARAMS geth --datadir data --mine --unlock 0x83a909262608c650bd9b0ae06e29d90d0f67ac5e --password config/passwd --keystore config/keystore --datadir data/ --rpc --rpcapi 'personal,db,eth,net,web3,txpool,miner,debug' --rpcvhosts '*' --rpccorsdomain '*'  --rpcaddr '0.0.0.0' --networkid 63819 --syncmode=full --gcmode=archive --nodiscover
else 
    echo [geth] Container exists, starting...
    docker start geth
fi

echo [script] waiting 10 seconds...
sleep 10

if [ ! "$(docker ps -q -f name=tesseracts)" ]; then
    echo [tesseracts] Container does not exist
    if [ "$(docker ps -aq -f status=exited -f name=tesseracts)" ]; then
        echo [tesseracts] Found old one, cleaning up
	docker rm tesseracts
    fi
    
    echo [geth] Starting new container...
    docker run -d --name tesseracts --link geth -v `pwd`/config:/config -v `pwd`/data:/data -p 8000:8000 -t adriamb/tesseracts:v0.4 -vvv --cfg /config/tesseracts.toml
else 
    echo [geth] Container exists, starting...
    docker start tesseracts
fi

