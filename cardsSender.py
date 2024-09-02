#!/usr/bin/env python3.8
import sys
import subprocess
import requests
import json
import base64
import time

#PROXY                = "https://devnet-gateway.elrond.com"
#API                  = "https://devnet-api.elrond.com"
#CHAIN_ID             = 'D'

PROXY                = "https://gateway.elrond.com"
API                  = "https://api.elrond.com"
CHAIN_ID             = '1'

COLLECTION_ID        = "BONCARDS-d1fb64"
QUANTITY_TO_SEND     = 90000
START_NONCE          = 85
FINAL_NONCE          = 168
ATTRIBUTES_SEPARATOR = ';'

SC_ADDRESS     = "erd1qqqqqqqqqqqqqpgqaww9ggpn5re7fze2dt4xls0xpch82dvh58sqpu76dr"
SENDER_ADDRESS = "erd1v07t9d57hsftstsj6ua8fppjxz2gh585erjpfg3y4kk39nap58sqy9zvsz"
PEM_PATH       = "../wallets/users/bon.pem"
BOOSTER_ID     = "BONPACKS-f0b549"
BOOSTER_NONCE  = 1

REFILL_FUNCTION = "refill"

LIST_INT_TO_RARITY = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]

def byte_to_str(value) :
	return str(value)[2:-1]

def parse_name(nft_request) :
	return nft_request["name"]

def parse_rarity(nft_request) :
	dico_rarity_to_int = {"Common": '0', "Uncommon": '1', "Rare": '2', "Epic": '3', "Legendary": '4'}

	base64_attributes = nft_request["attributes"]
	attributes = byte_to_str(base64.b64decode(base64_attributes))

	rarity = attributes.split("Rarity:")[1].split(ATTRIBUTES_SEPARATOR)[0].replace(' ', '')

	return dico_rarity_to_int[rarity]


def main() :

	sc_address_hex = "0x"+byte_to_str(subprocess.check_output(["erdpy", "wallet", "bech32", "--decode", SC_ADDRESS]))[:-2]

	print("Cards :")

	for i in range(START_NONCE, FINAL_NONCE+1) :
		nonce = (len(hex(i))%2)*'0' + hex(i)[2:]

		raw_request = json.loads(requests.get(API + "/collections/" + COLLECTION_ID + "/nfts?identifiers=" + COLLECTION_ID + '-' + nonce).text)[0]

		name   = parse_name(raw_request)
		rarity = parse_rarity(raw_request)

		print("\t\t[" + str(int(nonce, 16)-START_NONCE + 1) + "] Name   : " + name)

		print("\t\t\tID     : " + COLLECTION_ID)
		print("\t\t\tNonce  : " + nonce)
		print("\t\t\tRarity : " + LIST_INT_TO_RARITY[int(rarity)])

		subprocess.check_output(["erdpy", "contract", "call", SENDER_ADDRESS, "--recall-nonce",
			                     "--pem="+PEM_PATH, "--gas-limit=10000000", "--function=ESDTNFTTransfer",
			                     "--arguments", "0x"+COLLECTION_ID.encode('utf-8').hex(), str(int(nonce, 16)), str(QUANTITY_TO_SEND),
			                     sc_address_hex, "0x"+REFILL_FUNCTION.encode('utf-8').hex(), "0x"+BOOSTER_ID.encode('utf-8').hex(),
			                     hex(BOOSTER_NONCE), rarity, "--send", "--proxy="+PROXY, "--chain="+CHAIN_ID])

		time.sleep(20)
		
		print("\t\t***Transaction sent !***\n" + '*'*54)




if __name__ == "__main__" :
	main()
