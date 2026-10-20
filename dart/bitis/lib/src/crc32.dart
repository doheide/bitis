// CRC-32 helper (not part of the bitis wire format).
//
// Web-safe: the CRC register is always kept in 0 .. 2^32-1 and only combined
// with `^`, `&` and `>>>` (never `~` or `<<`), which give the same unsigned
// 32-bit results on the VM (64-bit ints), with dart2wasm (64-bit ints) and with
// dart2js (bitwise operators on unsigned 32-bit values).

import 'dart:typed_data';

/// 256-entry lookup table (1 KiB, built on first use).
final Uint32List _table = _makeTable();

Uint32List _makeTable() {
  final t = Uint32List(256);
  for (var i = 0; i < 256; i++) {
    var c = i;
    for (var k = 0; k < 8; k++) {
      c = (c & 1) != 0 ? (c >>> 1) ^ 0xEDB88320 : c >>> 1;
    }
    t[i] = c;
  }
  return t;
}

int _update(int c, List<int> bytes) {
  final t = _table;
  for (var i = 0; i < bytes.length; i++) {
    c = t[(c ^ bytes[i]) & 0xFF] ^ (c >>> 8);
  }
  return c;
}

/// CRC-32/ISO-HDLC of [bytes] (reflected polynomial `0xEDB88320`, init and
/// xorout `0xFFFFFFFF`; the CRC of zlib, Python `zlib.crc32`, Ethernet, PNG).
///
/// Only the low 8 bits of each element are used. The result is an unsigned
/// 32-bit value (`0 .. 0xFFFFFFFF`) on all platforms:
/// `crc32('123456789'.codeUnits) == 0xCBF43926`, `crc32([]) == 0`.
int crc32(List<int> bytes) => _update(0xFFFFFFFF, bytes) ^ 0xFFFFFFFF;

/// Incremental CRC-32/ISO-HDLC (same CRC as [crc32]).
///
/// The input may be split arbitrarily:
/// ```dart
/// final c = Crc32()..update(header)..update(payload);
/// final crc = c.value; // == crc32([...header, ...payload])
/// ```
class Crc32 {
  int _state = 0xFFFFFFFF;

  /// A CRC over the empty input.
  Crc32();

  /// Appends [bytes] (low 8 bits of each element) to the input; can be called
  /// any number of times.
  void update(List<int> bytes) {
    _state = _update(_state, bytes);
  }

  /// The CRC (`0 .. 0xFFFFFFFF`) of all bytes passed to [update] since
  /// creation or the last [reset]. Reading it does not change the state.
  int get value => _state ^ 0xFFFFFFFF;

  /// Same as [value].
  int finish() => value;

  /// Starts over with the empty input.
  void reset() {
    _state = 0xFFFFFFFF;
  }
}
