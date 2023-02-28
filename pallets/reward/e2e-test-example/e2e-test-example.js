import {MerkleTree} from 'merkletreejs';
import {keccak256AsU8a, decodeAddress} from '@polkadot/util-crypto';
import {WalkerImpl, encodeU128, encodeU64, encodeU32} from "@scale-codec/core";

/* 
   NOTE:
   - Ensure that the merkle trees are balanced (there are no single leaf branches) in order to prevent forgery attack. The total amount of leafs should be a power of 2 values (1,2,4,8... etc)
   - In the example I use 2 different hashing functions for the leaves and the branches in order to prevent second preimage attack
   - Leaves need to be encoded using SCALE in order to be validated by the Metaverse Network blockchain
*/

// SCALE Library - https://github.com/soramitsu/scale-codec-js-library
// You can also use https://github.com/paritytech/scale-ts

// (1) Get the whitelist data: pair of ClaimId, Balance / (ClassId, TokenId). Create index that maps each ClaimId to AccountId
const index = [
   [0, '5GuttyuDTejF1p6fzv1ffzxNKEnTWWJ4jCMwqcFfiwMj1bYh'], // sudo dev
   [1, '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'], // alice sudo
   [2, '5FHmnS7PYbm8MBNt33S9yuwiJEKkj3eT6ZcuQz1itW4fa3mA'], // blockchex
   [3, '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty'], // bob
   [4, '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y'], // charlie
   [5, '5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL'], // ferdie
   [6, '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy'], // dave
   [7, '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw'] // eva
];

const index_values = [
   [0, 100000000000000000000], 
   [1, 100000000000000000000], 
   [2, 100000000000000000000], 
   [3, 100000000000000000000], 
   [4, 100000000000000000000], 
   [5, 100000000000000000000], 
   [6, 100000000000000000000], 
   [7, 100000000000000000000], 
];

// (2) Encode reward data using SCALE
const scale_values = index_values.map(x => [...WalkerImpl.encode(BigInt(x[0]), encodeU64), ...WalkerImpl.encode(BigInt(x[1]), encodeU128)]);

// Encoding for NFT campaigns
//const scale_values = values.map(x => [...WalkerImpl.encode(BigInt(x[0]), encodeU64), ...WalkerImpl.encode(x[1], encodeU32)], , ...WalkerImpl.encode(BigInt(x[2]), encodeU64))

// (3) Build merkle tree using double hashing for leaves - use it ot call setRewardRoot together with passing the index as claim_index
// Make sure that you set the sortPairs option to true so the hashes will be calculated correctly.
// Make sure that your claim_index is sorted. Otherwise some accounts won't be able to claim immediately as the index is checked using binary search.
const leaves = scale_values.map(x => keccak256AsU8a(keccak256AsU8a(x)));
const tree = new MerkleTree(leaves, keccak256AsU8a, {sortPairs: true});

console.log("Merkle Tree: ", tree.toString());

// (4) Get proofs for each leaf - use them to call claimRewardRoot together with the relevant ClaimId
console.log(tree.getHexProof(leaves[0])); // example fopr first leaf
