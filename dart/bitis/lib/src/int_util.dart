// Internal helpers for web-safe integer handling.
//
// Under dart2js an `int` is an IEEE double (exact up to 2^53) and bitwise
// operators work on 32-bit values only. Everything in this library therefore
// represents values of up to 64 bits as two 32-bit halves `(lo, hi)` (each in
// `0 .. 2^32-1`) and only applies bitwise operators to values below 2^32 with
// shift distances below 32. Combining halves into larger numbers is done with
// exact floating point arithmetic (`hi * 2^32 + lo` for results < 2^53).

import 'package:fixnum/fixnum.dart';

/// 2^32 as an int (exact on all platforms).
const int two32 = 4294967296;

/// The largest integer `n` such that `n` and `n + 1` are both exactly
/// representable as a double (JavaScript `Number.MAX_SAFE_INTEGER`).
const int maxSafeInteger = 9007199254740991;

/// `pow2[k] == 2^k` for `0 <= k <= 53` (computed without shifting by >= 32).
final List<int> pow2 = List<int>.unmodifiable(
  List<int>.generate(54, _pow2Slow),
);

int _pow2Slow(int k) {
  var r = 1;
  for (var i = 0; i < k; i++) {
    r *= 2;
  }
  return r;
}

/// `lowMask[k] == 2^k - 1` for `0 <= k <= 32`.
final List<int> lowMask = List<int>.unmodifiable(
  List<int>.generate(33, (k) => pow2[k] - 1),
);

/// Number of significant bits of the unsigned 64-bit value `(lo, hi)`.
int bitLengthU64(int lo, int hi) => hi != 0 ? 32 + hi.bitLength : lo.bitLength;

/// Splits a non-negative [v] (`v <= 2^53` on the web, `< 2^63` on native)
/// into 32-bit halves.
(int, int) splitInt(int v) {
  final hi = v ~/ two32;
  return (v - hi * two32, hi);
}

/// Combines 32-bit halves into an int. Only exact if the value is < 2^53.
int joinInt(int lo, int hi) => hi * two32 + lo;

/// Splits the two's complement bit pattern of [v] into 32-bit halves.
(int, int) splitInt64(Int64 v) {
  final b = v.toBytes(); // little endian, web-safe
  final lo = b[0] + b[1] * 0x100 + b[2] * 0x10000 + b[3] * 0x1000000;
  final hi = b[4] + b[5] * 0x100 + b[6] * 0x10000 + b[7] * 0x1000000;
  return (lo, hi);
}

/// Builds the Int64 with the two's complement bit pattern `(lo, hi)`.
Int64 joinInt64(int lo, int hi) => Int64.fromInts(hi, lo);

final BigInt _mask32 = BigInt.from(0xFFFFFFFF);

/// Splits a non-negative BigInt with at most 64 bits into 32-bit halves.
(int, int) splitBigInt(BigInt v) =>
    ((v & _mask32).toInt(), ((v >> 32) & _mask32).toInt());

/// Builds a non-negative BigInt from 32-bit halves.
BigInt joinBigInt(int lo, int hi) => (BigInt.from(hi) << 32) + BigInt.from(lo);

/// Width of the Rust/C++ storage type of a signed field with [bits] value
/// bits (spec 2.3): 16, 32 or 64.
int signedBaseWidth(int bits) => bits <= 16 ? 16 : (bits <= 32 ? 32 : 64);

/// Maximum number of significant magnitude bits of a signed field with
/// [bits] value bits: `min(X, B - 1)`. The valid range of `int_X` is
/// `|v| <= 2^maxMagnitudeBits(X) - 1` (spec 2.3).
int signedMagnitudeBits(int bits) {
  final b = signedBaseWidth(bits) - 1;
  return bits < b ? bits : b;
}
