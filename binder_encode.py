import sys

if len(sys.argv) < 2:
	print("no args")
	exit(1)
arg = sys.argv[1]

encoded = []
i_encoded = 0
for i in range(len(arg) * 2 - 1):
	if i % 2 == 0:
		encoded.append(arg[i_encoded])
		i_encoded += 1
	else:
		encoded.append("0")
encoded = ''.join(encoded)
print(encoded)

if len(sys.argv) > 2:
	name = sys.argv[2]
else:
	name = "tmp"

print(f"char {name}[] = {{", end='')
len_enc = len(encoded)
for i, c in enumerate(encoded):
	if i % 15 == 0:
		print("\n    ", end='')
	if c == '0':
		print(f"0x0, ", end='')
	else:
		if i == len_enc - 1:
			print(f"'{c}'", end='')
		else:
			print(f"'{c}', ", end='')

print("};")
print(f"size_t {name}_len = {len(encoded)};")