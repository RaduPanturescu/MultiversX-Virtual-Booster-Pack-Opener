#!/usr/bin/env python3.8
import sys
import requests

RARITY_NAMES = ["Common    ", "Uncommon  ", "Rare      ", "Epic      ", "Legendary "]

def get_nbr_res() :
	nbr_res = 0
	for e in sys.argv[2: ] :
		if (e == '{' or e == "\"\",") :
			nbr_res += 1
	return nbr_res

def main() :
	if len(sys.argv) == 1 :
		print("Error : Missing argument ")
		return -1
	if (sys.argv[1] == "--parseInt") :
		if '"number":' in sys.argv :
			print(sys.argv[sys.argv.index('"number":')+1])
		else :
			print(0)
	elif (sys.argv[1] == "--parseProbabilities") :
		if (sys.argv[2] == "-spaces") :
			spaces = 32*' '
			print("              -rarities       :")
		else :
			spaces = ''
		probabilities = sys.argv[sys.argv.index('"hex":')+1][1:-2]
		for i in range(5) :
			print(spaces + "-" + RARITY_NAMES[i] + ": " + str(int(probabilities[i*8:(i+1)*8], 16)/1000)  + "%")
	elif (sys.argv[1] == "--parseGuarantees") :
		if (sys.argv[2] == "-spaces") :
			spaces = 32*' '
			print("              -guarantees     :")
		else :
			spaces = ''
		if get_nbr_res() > 0 :
			startIndex = sys.argv.index('[') + 1
			values = list()
			for i in range(len(sys.argv[startIndex:])) :
				if sys.argv[startIndex+i] == "\"\"," :
					values.append(0)
				if sys.argv[startIndex+i] == "\"number\":" :
					values.append(int(sys.argv[startIndex+i+1]))
				if len(values) == 2 :
					print(spaces + "-" + RARITY_NAMES[values[0]] + ": " + str(values[1]))
					values = list()
		else :
			print(spaces + "No guarantees")
	elif (sys.argv[1] == "--parseConstraints") :
		if (sys.argv[2] == "-spaces") :
			spaces = 32*' '
			print("              -constraints    :")
		else :
			spaces = ''
		if '"hex":' in sys.argv and int(sys.argv[sys.argv.index('"hex":')+1][1:-2]) != 0 :
			constraints = sys.argv[sys.argv.index('"hex":')+1][1:-2]
			for i in range(5) :
				print(spaces + "-" + RARITY_NAMES[i] + ": " + str(int(constraints[i*16:(i+1)*16], 16)))
		else :
			print(spaces + "No constraints")
	elif (sys.argv[1] == "--parseCards") :
		sc_address = sys.argv[2]
		api = sys.argv[3]
		rarity = RARITY_NAMES[int(sys.argv[6])].replace(' ', '')
		print("        " + rarity + "s cards :")
		if get_nbr_res() > 0 :
			startIndex = sys.argv.index('[') + 1
			count = 1
			for i in range(len(sys.argv[startIndex:])) :
				if sys.argv[startIndex+i] == "\"\"," :
					print("                        [" + str(count) + "] -ID       : ")
					print("                         " + len(str(count))*' ' + "  -nonce    : 0")
					#print("                         " + len(str(count))*' ' + "  -quantity : 0")
					print("******************************************************")
					count += 1
				if sys.argv[startIndex+i] == "\"hex\":" :
					value = sys.argv[startIndex+i+1][1:-2]
					ID = bytes.fromhex(value[:value.index("2d")+14]).decode('utf-8')[4:]
					nonce = str(int(value[value.index("2d")+14:value.index("2d")+14+16], 16))

					raw_request = requests.get(api + "/accounts/" + sc_address + "/nfts/" + ID + '-' + len(hex(int(nonce))[2:])%2*'0' + hex(int(nonce))[2:]).text
					name = raw_request.split("name\":\"")[1].split("\"")[0]
					quantity = raw_request.split("balance\":\"")[1].split("\"")[0]
					print("                        [" + str(count) + "] -name     : " + name)
					print("                         " + len(str(count))*' ' + "  -ID       : " + ID)
					print("                         " + len(str(count))*' ' + "  -nonce    : " + nonce)
					print("                         " + len(str(count))*' ' + "  -rarity   : " + rarity)
					print("                         " + len(str(count))*' ' + "  -quantity : " + quantity)
					print("******************************************************")
					count += 1
		else :
			print("        No cards")
	elif (sys.argv[1] == "--parseTokenName") :
		ID = bytes.fromhex(sys.argv[2][2:]).decode('utf-8')
		nonce = (len(sys.argv[3])%2)*'0' + sys.argv[3]
		api = sys.argv[4]
		raw_request = requests.get(api + "/collections/" + ID + "/nfts?identifiers=" + ID + "-" + nonce).text
		name = raw_request.split("name\":\"")[1].split("\"")[0]
		print(name)
	else :
		print("Error : Parsing not implemented")
		return -1

	return 0






if __name__ == "__main__" :
	main()