
//const StandardMerkleTree = require("@openzeppelin/merkle-tree");
const MerkleTree = require ('merkletreejs');
const keccak256AsU8a = require ('@polkadot/util-crypto');


// (1) Get data and encode it using SCALE 
// SCALE Libraries:
// https://github.com/paritytech/scale-ts
// https://github.com/soramitsu/scale-codec-js-library

const values = [
   [2, 10],
   [3, 25],
   [4, 50],
   [5, 75]
];

const bob_leaf_array = new Uint8Array([2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const charlie_leaf_array = new Uint8Array([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const donna_leaf_array = new Uint8Array([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
const eva_leaf_array = new Uint8Array([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

const scale_values = [
   bob_leaf_array, 
   charlie_leaf_array, 
   donna_leaf_array, 
   eva_leaf_array
];

// (2) Build merkle tree using double hashing for leaves
//const tree = StandardMerkleTree.of(values, ["uint256", "uint256"]);
const leaves = scale_values.map(x => keccak256AsU8a(keccak256AsU8a(x)));
const tree = new MerkleTree(leaves, keccak256AsU8a);

// (3) Output the tree 
console.log("Merkle Tree: ", tree.toString());