import { MerkleTree} from 'merkletreejs';
import {keccak256AsU8a} from '@polkadot/util-crypto';
import {WalkerImpl, createTupleEncoder, createArrayEncoder, encodeU128, encodeU64, encodeU32} from "@scale-codec/core";

/* 
   NOTE:
   - Ensure that the merkle trees are balanced (there are no single leaf branches) in order to prevent forgery attack. The total amount of leafs should be a power of 2 values (1,2,4,8... etc)
   - In the example I use 2 different hashing functions for the leaves and the branches in order to prevent second preimage attack
   - Leaves need to be encoded using SCALE in order to be validated by the Metaverse Network blockchain
*/

// (1) Get the whitelist data: pair of AccountId, Balance / (ClassId, TokenId)
const values = [
   [2, 10],
   [3, 25],
   [4, 50],
   [5, 75]
];


// SCALE Libraries:
// https://github.com/paritytech/scale-ts
// https://github.com/soramitsu/scale-codec-js-library

// Simple example of SCALE encoding for a leaf 
const bob = [2,10];
const bob_output = [...WalkerImpl.encode(BigInt(bob[0]), encodeU128), ...WalkerImpl.encode(BigInt(bob[1]), encodeU128)];
console.log("Bob SCALE: ", bob_output);

// TEST DATA (VALID SCALE OUTPUT)

const bob_leaf_array = new Uint8Array([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
console.log("Is Bob SCALE encoding correct? ", bob_output.toString() === bob_leaf_array.toString());

const charlie_leaf_array = new Uint8Array([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const donna_leaf_array = new Uint8Array([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const eva_leaf_array = new Uint8Array([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const charlie2_leaf_array = new Uint8Array([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 256, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const donna2_leaf_array = new Uint8Array([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 501, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const eva2_leaf_array = new Uint8Array([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 751, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

const scale_values_valid = [
   bob_leaf_array, 
   charlie_leaf_array, 
   donna_leaf_array, 
   eva_leaf_array,
   //charlie2_leaf_array,
  // donna2_leaf_array
];

// (2) Encode data using SCALE
const scale_values = values.map(x => [...WalkerImpl.encode(BigInt(x[0]), encodeU128), ...WalkerImpl.encode(BigInt(x[1]), encodeU128)])

// Encoding for NFT-based input
//const scale_values = values.map(x => [...WalkerImpl.encode(BigInt(x[0]), encodeU128), ...WalkerImpl.encode(BigInt(x[1]), encodeU32)], , ...WalkerImpl.encode(BigInt(x[2]), encodeU64))

// (3) Build merkle tree using double hashing for leaves - use it ot call setRewardRoot
//const tree = StandardMerkleTree.of(values, ["uint256", "uint256"]);
const leaves = scale_values.map(x => keccak256AsU8a(keccak256AsU8a(x)));
const tree = new MerkleTree(leaves, keccak256AsU8a);

console.log("Merkle Tree: ", tree.toString());

// (4) Get proofs for each leaf - use them to call claimRewardRoot
const bob_proof = tree.getHexProof(leaves[0]);
const charlie_proof = tree.getHexProof(leaves[1]);
const donna_proof = tree.getHexProof(leaves[2]);
const eva_proof = tree.getHexProof(leaves[3]);

console.log("Bob Proof Array: ", bob_proof);
console.log("Charlie Proof Array: ", charlie_proof);
console.log("Donna Proof Array: ", donna_proof);
console.log("Eva Proof Array: ", eva_proof);