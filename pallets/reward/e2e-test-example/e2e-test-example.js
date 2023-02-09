import { MerkleTree} from 'merkletreejs';
import {hexToU8a} from '@polkadot/util';
import {keccak256AsU8a, encodeAddress, decodeAddress} from '@polkadot/util-crypto';
import {WalkerImpl, createTupleEncoder, createArrayEncoder, encodeU128, encodeU64, encodeU32, encodeStr} from "@scale-codec/core";

/* 
   NOTE:
   - Ensure that the merkle trees are balanced (there are no single leaf branches) in order to prevent forgery attack. The total amount of leafs should be a power of 2 values (1,2,4,8... etc)
   - In the example I use 2 different hashing functions for the leaves and the branches in order to prevent second preimage attack
   - Leaves need to be encoded using SCALE in order to be validated by the Metaverse Network blockchain
*/

// SCALE Libraries:
// https://github.com/paritytech/scale-ts
// https://github.com/soramitsu/scale-codec-js-library

// Simple example of SCALE encoding for a leaf 
const bob = [2,10];
//const bob_address = encodeAddress(WalkerImpl.encode(BigInt(bob[0]), encodeU128));
//const bob_output_address = [...decodeAddress(bob_address), ...WalkerImpl.encode(BigInt(bob[1]), encodeU128)];
//console.log("Bob SCALE: ", bob_output);

// TEST DATA (VALID SCALE OUTPUT)
const bob_leaf_array = new Uint8Array([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const charlie_leaf_array = new Uint8Array([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const donna_leaf_array = new Uint8Array([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const eva_leaf_array = new Uint8Array([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

const scale_values_valid = [
   bob_leaf_array, 
   charlie_leaf_array, 
   donna_leaf_array, 
   eva_leaf_array,
];

// (1) Get the whitelist data: pair of AccountId, Balance / (ClassId, TokenId)
const values = [
   ['5FpqLqqbFyYWgYtgQS11HvTkaripk1nPFFti6CwDaMj8cSvu', 10],
   ['5EUXjqNx3Rsh3wtDJAPBzEvJdGVD3QmxmMUjrfARNr4uh7pq', 25],
   ['5Dqy8KtwmGJd6Tkar8Va3Uw7xvX4RQAhrygUk3T8vUxDXf2a', 50],
   ['5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY', 75]
];

// (2) Decode the address string and encode reward data using SCALE
const scale_values = values.map(x => [...decodeAddress(x[0]), ...WalkerImpl.encode(BigInt(x[1]), encodeU128)]);

// Alternatively the address can be encoded as string. !!! Need to test and confirm if that would produce valid leaf hashes
// const scale_values = values.map(x => [...WalkerImpl.encode(x[0], encodeStr), ...WalkerImpl.encode(BigInt(x[1]), encodeU128)]);

// Encoding for NFT-based input
//const scale_values = values.map(x => [...decodeAddress(x[0]), ...WalkerImpl.encode(x[1], encodeU32)], , ...WalkerImpl.encode(BigInt(x[2]), encodeU64))

// (3) Build merkle tree using double hashing for leaves - use it ot call setRewardRoot
//const tree = StandardMerkleTree.of(values, ["uint256", "uint256"]);
const leaves = scale_values.map(x => keccak256AsU8a(keccak256AsU8a(x)));
const tree = new MerkleTree(leaves, keccak256AsU8a);

console.log("Merkle Tree: ", tree.toString());

// (4) Get proofs for each leaf - use them to call claimRewardRoot
const bob_proof = tree.getHexProof(leaves[0]).map(x => x.toString().substring(2));
const charlie_proof = tree.getHexProof(leaves[1]).map(x => x.toString().substring(2));
const donna_proof = tree.getHexProof(leaves[2]).map(x => x.toString().substring(2));
const eva_proof = tree.getHexProof(leaves[3]).map(x => x.toString().substring(2));

console.log("Bob Proof Array: ", bob_proof);
console.log("Charlie Proof Array: ", charlie_proof);
console.log("Donna Proof Array: ", donna_proof);
console.log("Eva Proof Array: ", eva_proof);