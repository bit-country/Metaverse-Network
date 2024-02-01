import { encodeAddress, blake2AsHex } from '@polkadot/util-crypto';

function getDefaultSubstrateAddress(evmAddress, prefix) {
    const addressBytes = Buffer.from(evmAddress.slice(2), 'hex');
    const prefixBytes = Buffer.from('evm:');
    const convertBytes = Uint8Array.from(Buffer.concat([ prefixBytes, addressBytes ]));
    const finalAddressHex = blake2AsHex(convertBytes, 256);
    return encodeAddress(finalAddressHex, prefix)
}

console.log("The default Substrate address for " + process.argv[2] + " is: " + getDefaultSubstrateAddress(process.argv[2], process.argv[3]));
