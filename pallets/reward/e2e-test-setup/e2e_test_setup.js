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
      ['5C7LYpP2ZH3tpKbvVvwiVe54AapxErdPBbvkYhe6y9ZBkqWt', 10], // bob
      ['5C8etthaGJi5SkQeEDSaK32ABBjkhwDeK9ksQCTLEGM3EH14', 25], // charlie
      ['5C9yEy27yLNG5BDMxVwS8RyGBneZB1ouShazFhGZVP8thK5z', 50], // donna
      ['5CBHb3LfgN2Shc25gnSHwpvNCPZMe6QAaFR77C5nkVvkAK1o', 75] // eva
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
