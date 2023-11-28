# zkBitcoin

## Set up Bitcoin CLI

```shell
brew install bitcoin
```

## Set up bitcoind

Both users and nodes need access to Bitcoin.

1. download latest release on https://bitcoincore.org/en/releases/ (for example `wget https://bitcoincore.org/bin/bitcoin-core-25.1/bitcoin-25.1-x86_64-linux-gnu.tar.gz`)
2. you can run bitcoind with `./bin/bitcoind -testnet -server=1 -rpcuser=root -rpcpassword=hello`
    - or you can set these in `bitcoin.conf` and copy that file in `~/.bitcoin/bitcoin.conf`
3. you can query it for testing with `./bin/bitcoin-cli -rpcport=18332 -rpcuser=root -rpcpassword=hellohello getblock`

you can also use curl to query it:

```console
curl --user root --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "getblock", "params": ["00000000c937983704a73af28acdec37b049d214adbda81d7e2a3dd146f6ed09"]}' -H 'content-type: text/plain;' http://127.0.0.1:18332/
```

If you want to expose this to the internet, you can setup a reverse proxy like nginx. This is the config I use (in `/etc/nginx/sites-enabled/bitcoind-proxy.conf`):

```
server {
    listen 18331;
    server_name _;

    location / {
        proxy_pass http://127.0.0.1:18332;
    }
}
```

## Our own bitcoind

I'm running one on digitalocean at [146.190.33.39](http://146.190.33.39)

You can query it with:

```console
curl --user root:hellohello --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "getblockchaininfo", "params": []}' -H 'content-type: text/plain;' http://146.190.33.39:18331
```

### Our wallet

I created a wallet called "mywallet" via:

```console
bitcoin-cli -testnet -rpcconnect=146.190.33.39 -rpcport=18331 -rpcuser=root -rpcpassword=hellohello createwallet mywallet
```

you can get information about wallets in general using:

```console
bitcoin-cli -testnet -rpcconnect=146.190.33.39 -rpcport=18331 -rpcuser=root -rpcpassword=hellohello getwalletinfo
```

and obtain a new public key via:

```console
bitcoin-cli -testnet -rpcconnect=146.190.33.39 -rpcport=18331 -rpcuser=root -rpcpassword=hellohello getnewaddress
```

I believe we can switch wallets by using the `loadwallet` command.

* `tb1q5pxn428emp73saglk7ula0yx5j7ehegu6ud6ad` <-- I put some bitcoins in there from [this faucet](https://bitcoinfaucet.uo1.net/send.php) (you can see the wallet on [this explorer](https://blockstream.info/testnet/address/tb1q5pxn428emp73saglk7ula0yx5j7ehegu6ud6ad) where 141 satoshis were paid as fee)
* `tb1q6nkpv2j9lxrm6h3w4skrny3thswgdcca8cx9k6
`
