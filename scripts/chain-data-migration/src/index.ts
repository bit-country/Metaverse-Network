import {ApiPromise, WsProvider} from '@polkadot/api';
import {hexToString, hexToBigInt, hexToNumber} from '@polkadot/util';
import {encodeAddress, decodeAddress} from '@polkadot/util-crypto'
// Writing to JSON. To Do: replace with DB  reading and writing
import * as fs from "fs";

const CONTINUUM_ENDPOINT = 'wss://continuum-rpc-1.metaverse.network/wss';
const PIONEER_ENDPOINT = 'wss://pioneer-rpc-3.bit.country/wss';
const PIONEER_PREFIX = 268;

export async function fetchPioneerNftData() {
  const pioneerApi = await ApiPromise.create({
    provider: new WsProvider(PIONEER_ENDPOINT),
  });
  console.log('☑️ Connected to Pioneer Endpoint');
  const pioneerCollectionData =
    await pioneerApi.query.nft.groupCollections.entries();
  console.log('☑️ Fetched Pioneer Collections Data');
  const pioneerClassData = await pioneerApi.query.ormlNFT.classes.entries();
  console.log('☑️ Fetched Pioneer Classes Data');
  const pioneerTokenData = await pioneerApi.query.ormlNFT.tokens.entries();
  console.log('☑️ Fetched Pioneer Tokens Data');
  if (!process.env.ENABLE_JSON_OUTPUT) {
      console.log('JSON output disabled. Printing nft information');
      console.log('================================================================');

      pioneerCollectionData.forEach(
          ([
               {
                   args: [collectionId],
               },
               value,
           ]) => {
              console.log(`CollectionId: ${collectionId}`);
              let collectionInfo = JSON.parse(JSON.stringify(value));
              console.log(`  Name: ${hexToString(collectionInfo.name)}`);
              console.log(`  Properties: ${hexToString(collectionInfo.properties)}`);
          }
      );
      console.log('================================================================');

      pioneerClassData.forEach(
          ([
               {
                   args: [classId],
               },
               value,
           ]) => {
              console.log(`ClassId: ${classId}`);
              let classInfo = JSON.parse(JSON.stringify(value));
              console.log(`  Metadata: ${classInfo.metadata.toString()}`);
              console.log(`  TotallIssuance: ${classInfo.totalIssuance.toString()}`);
              let classOwner = encodeAddress(decodeAddress(classInfo.owner), PIONEER_PREFIX)
              console.log(`  Owner: ${classOwner}`);
              let classData = JSON.parse(JSON.stringify(classInfo.data));
              console.log(`  Data:`);
              console.log(`     Deposit: ${hexToBigInt(classData.deposit)}`);
              console.log(`     TokenType: ${classData.tokenType.toString()}`);
              console.log(`     IsLocked: ${classData.isLocked.toString()}`);
              console.log(`     RoyaltyFee: ${classData.royaltyFee.toString()}`);
              console.log(`     MintLimit: ${classData.mintLimit}`);
              console.log(`     TotalMintedTokens: ${classData.totalMintedTokens.toString()}`);
              let classCollectionType = JSON.parse(JSON.stringify(classData.collectionType));
              if (classCollectionType.collectable === null) {
                  console.log(`     CollectionType: Collectable`);
              }
              let classAttributes = JSON.parse(JSON.stringify(classData.attributes));
              if (classAttributes !== null) {
                  // TODO: better attributes parsing
                  console.log(`     Attributes:`);
                  console.log(`        ${hexToString('0x43617465676f72793a')} ${hexToString(classAttributes['0x43617465676f72793a'])}`);
                  console.log(`        ${hexToString('0x4d657461766572736549643a')} ${hexToString(classAttributes['0x4d657461766572736549643a'])}`);
              }
              else {
                  console.log(`     Attributes: ${JSON.stringify(classData.attributes)}`);
              }
          }
      );
      console.log('================================================================');

      pioneerTokenData.forEach(
          ([
               {
                   args: [classId,tokenId],
               },
               value,
           ]) => {
              let tokenInfo = JSON.parse(JSON.stringify(value));
              let tokenOwner = encodeAddress(decodeAddress(tokenInfo.owner), PIONEER_PREFIX)
              let tokenData = JSON.parse(JSON.stringify(tokenInfo.data));
              let tokenAttributes = JSON.parse(JSON.stringify(tokenData.attributes));
              console.log(`ClassId: ${classId}`);
              console.log(`TokenId: ${tokenId}`);
              console.log(`  Metadata: ${tokenInfo.metadata.toString()}`);
              console.log(`  Owner: ${tokenOwner}`);
              console.log(`  Data:`);
              console.log(`     Deposit: ${hexToBigInt(tokenData.deposit)}`);
              console.log(`     IsLocked: ${tokenData.isLocked.toString()}`);
              if (tokenAttributes !== null) {
                  // TODO: better attributes parsing
                  console.log(`     Attributes:`);
                  console.log(`        ${hexToString('0x436f6f7264696e6174653a')} ${tokenAttributes['0x436f6f7264696e6174653a']}`);
                  console.log(`        ${hexToString('0x4d657461766572736549643a')} ${hexToNumber(tokenAttributes['0x4d657461766572736549643a'])}`);
              }
              else {
                  console.log(`     Attributes: ${JSON.stringify(tokenData.attributes)}`);
              }
          }
      );
      console.log('================================================================');

      //printCollectionData(pioneerCollectionData);
      //printClassData(pioneerClassData);
      //printTokenData(pioneerTokenData);
  }
  else {
      // TODO: Decode Collection Id-s
      fs.writeFile('./data/pioneer-collections.json', JSON.stringify(pioneerCollectionData, null, '\t'), (err) => {
          // In case of a error throw err.
          if (err) {
              console.log('🙅‍♂️ Failed to store nft collections data!');
              throw err;
          }
      })
      console.log('☑️ Stored Pioneer collections data in pioneer-collections.json');

      fs.writeFile('./data/pioneer-classes.json', JSON.stringify(pioneerClassData, null , '\t'), (err) => {
          // In case of a error throw err.
          if (err) {
              console.log('🙅‍♂️ Failed to store nft class data!');
              throw err;
          }
      })
      console.log('☑️ Stored Pioneer classes data in pioneer-classes.json');

      fs.writeFile('./data/pioneer-tokens.json', JSON.stringify(pioneerTokenData, null , '\t'), (err) => {
          // In case of a error throw err.
          if (err) {
              console.log('🙅‍♂️ Failed to store nft tokens data!');
              throw err;
          }
      })
      console.log('☑️ Stored Pioneer tokens data in pioneer-tokens.json');
  }
  process.exit();
  return
}

export async function calculateContinuumClassValues() {
  const continuumApi = await ApiPromise.create({
    provider: new WsProvider(CONTINUUM_ENDPOINT),
  });
  console.log('☑️ Connected to Continuum API');
  return true;
}
/*
function printCollectionData(collectionData: JSON) {
    console.log('================================================================');

    collectionData.forEach(
        ([
             {
                 args: [collectionId],
             },
             value,
         ]) => {
            console.log(`CollectionId: ${collectionId}`);
            let collectionInfo = JSON.parse(JSON.stringify(value));
            console.log(`  Name: ${hexToString(collectionInfo.name)}`);
            console.log(`  Properties: ${hexToString(collectionInfo.properties)}`);
        }
    );
    console.log('================================================================');
    return;
}

function printClassData(classData: JSON) {
    classData.forEach(
        ([
             {
                 args: [classId],
             },
             value,
         ]) => {
            console.log(`ClassId: ${classId}`);
            let classInfo = JSON.parse(JSON.stringify(value));
            console.log(`  Metadata: ${classInfo.metadata.toString()}`);
            console.log(`  TotallIssuance: ${classInfo.totalIssuance.toString()}`);
            let classOwner = encodeAddress(decodeAddress(classInfo.owner), PIONEER_PREFIX)
            console.log(`  Owner: ${classOwner}`);
            let classData = JSON.parse(JSON.stringify(classInfo.data));
            console.log(`  Data:`);
            console.log(`     Deposit: ${hexToBigInt(classData.deposit)}`);
            console.log(`     TokenType: ${classData.tokenType.toString()}`);
            console.log(`     IsLocked: ${classData.isLocked.toString()}`);
            console.log(`     RoyaltyFee: ${classData.royaltyFee.toString()}`);
            console.log(`     MintLimit: ${classData.mintLimit}`);
            console.log(`     TotalMintedTokens: ${classData.totalMintedTokens.toString()}`);
            let classCollectionType = JSON.parse(JSON.stringify(classData.collectionType));
            if (classCollectionType.collectable === null) {
                console.log(`     CollectionType: Collectable`);
            }
            let classAttributes = JSON.parse(JSON.stringify(classData.attributes));
            // TODO: better attributes parsing
            console.log(`     Attributes:`);
            console.log(`        ${hexToString('0x43617465676f72793a')} ${hexToString(classAttributes['0x43617465676f72793a'])}`);
            console.log(`        ${hexToString('0x4d657461766572736549643a')} ${hexToString(classAttributes['0x4d657461766572736549643a'])}`);
        }
    );
    console.log('================================================================');
    return;
}

function printTokenData(tokenData: JSON) {
    console.log(`Token Data: ${tokenData}`);
    return;
}
*/
fetchPioneerNftData();
