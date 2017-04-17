#include <stddef.h>

typedef int i32;
typedef unsigned char u8;
typedef unsigned long u64;
typedef size_t usize;

#define SYM "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"

static const u8 symbols[256] = SYM SYM SYM SYM;

static const u8 values[256] = {
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 62, 99, 99, 99, 63,
	52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 99, 99, 99, 99, 99, 99,
	99,  0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14,
	15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 99, 99, 99, 99, 99,
	99, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
	41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
	99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 99,
};

#define CONCAT(X, Y) X ## _ ## Y
#define SUFFIX(X, Y) CONCAT(X, Y)

void SUFFIX(encode_seq, COMPILER)
(u8 const* restrict input, usize len, u8* restrict output)
{
	for (; len; len -= 3) {
		*output++ = symbols[(u8)(input[0] >> 2)];
		*output++ = symbols[(u8)(input[0] << 4 | input[1] >> 4)];
		*output++ = symbols[(u8)(input[1] << 2 | input[2] >> 6)];
		*output++ = symbols[(u8)input[2]];
		input += 3;
	}
}

void SUFFIX(encode_par, COMPILER)
(u8 const* restrict input, usize len, u8* restrict output)
{
	for (; len; len -= 3) {
		u64 x = (u64)input[0] << 16 | (u64)input[1] << 8 | input[2];
		output[0] = symbols[(u8)(x >> 18)];
		output[1] = symbols[(u8)(x >> 12)];
		output[2] = symbols[(u8)(x >> 6)];
		output[3] = symbols[(u8)x] ;
		input += 3;
		output += 4;
	}
}

i32 SUFFIX(decode_seq, COMPILER)
(u8 const* restrict input, usize len, u8* restrict output)
{
	for (; len; len -= 4) {
		if (values[input[0]] >= 64) return -1;
		if (values[input[1]] >= 64) return -1;
		*output++ = values[input[0]] << 2 | values[input[1]] >> 4;
		if (values[input[2]] >= 64) return -1;
		*output++ = values[input[1]] << 4 | values[input[2]] >> 2;
		if (values[input[3]] >= 64) return -1;
		*output++ = values[input[2]] << 6 | values[input[3]];
		input += 4;
	}
	return 0;
}

i32 SUFFIX(decode_par, COMPILER)
(u8 const* restrict input, usize len, u8* restrict output)
{
	for (; len; len -= 4) {
		if (values[input[0]] & values[input[1]] & values[input[2]]
		    & values[input[3]] & 0xc0) {
			return -1;
		}
		u64 x = values[input[0]] << 18 | values[input[1]] << 12
			| values[input[2]] << 6 | values[input[3]];
		output[0] = x >> 16;
		output[1] = x >> 8;
		output[2] = x;
		input += 4;
		output += 3;
	}
	return 0;
}
