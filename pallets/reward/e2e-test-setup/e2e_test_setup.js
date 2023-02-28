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


// (1) Get the whitelist data: pair of AccountId,  Balance / (ClassId, TokenId).
const values  = [ 
      ['5GuttyuDTejF1p6fzv1ffzxNKEnTWWJ4jCMwqcFfiwMj1bYh', 100000000000000000000], // sudo dev
      ['5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY', 100000000000000000000], // alice sudo
      ['5FHmnS7PYbm8MBNt33S9yuwiJEKkj3eT6ZcuQz1itW4fa3mA', 100000000000000000000], // blockchex
      ['5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', 100000000000000000000], // bob
      ['5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y', 100000000000000000000], // charlie
      ['5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL', 100000000000000000000], // freddie
      ['5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy', 100000000000000000000], // dave
      ['5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw', 100000000000000000000] // eva
];

// (2) Decode address and encode reward using SCALE
const scale_values = values.map(x => [...decodeAddress(x[0]), ...WalkerImpl.encode(BigInt(x[1]), encodeU128)]);

// Encoding for NFT campaigns
//const scale_values = values.map(x => [...decodeAddress(x[0]), encodeU64), ...WalkerImpl.encode(x[1], encodeU32), ...WalkerImpl.encode(BigInt(x[2]), encodeU64)])

// (3) Build merkle tree using double hashing for leaves - use it ot call setRewardRoot
const leaves = scale_values.map(x => keccak256AsU8a(keccak256AsU8a(x)));
const tree = new MerkleTree(leaves, keccak256AsU8a, {sortPairs: true});

console.log("Merkle Tree: ", tree.toString());

console.log("Merkle Tree Root: ", tree.getHexRoot().toString());
// (4) Get proofs for each leaf - use them to call claimRewardRoot

console.log(tree.getHexProof(leaves[7]));
