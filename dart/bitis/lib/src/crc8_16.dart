// CRC-8/AUTOSAR, CRC-16/IBM-3740 and transmission CRC helpers (not part of
// the bitis wire format).
//
// Web-safe: the CRC registers stay below 2^24 (`<<` is only applied to values
// of at most 16 bits) and are masked with `&`, so the VM, dart2wasm and
// dart2js give the same unsigned results.

import 'dart:typed_data';

/// 256-entry CRC-8/AUTOSAR table (poly 0x2F), built on first use.
final Uint8List _table8 = _makeTable8();

/// 256-entry CRC-16/IBM-3740 table (poly 0x1021), built on first use.
final Uint16List _table16 = _makeTable16();

/// Largest frame length (without the CRC) protected by CRC-8.
const int _crc8MaxN = 14;

Uint8List _makeTable8() {
  final t = Uint8List(256);
  for (var i = 0; i < 256; i++) {
    var c = i;
    for (var k = 0; k < 8; k++) {
      c = (c & 0x80) != 0 ? ((c << 1) ^ 0x2F) & 0xFF : (c << 1) & 0xFF;
    }
    t[i] = c;
  }
  return t;
}

Uint16List _makeTable16() {
  final t = Uint16List(256);
  for (var i = 0; i < 256; i++) {
    var c = i << 8;
    for (var k = 0; k < 8; k++) {
      c = (c & 0x8000) != 0 ? ((c << 1) ^ 0x1021) & 0xFFFF : (c << 1) & 0xFFFF;
    }
    t[i] = c;
  }
  return t;
}

int _update8(int c, List<int> bytes, int start, int end) {
  final t = _table8;
  for (var i = start; i < end; i++) {
    c = t[(c ^ bytes[i]) & 0xFF];
  }
  return c;
}

int _update16(int c, List<int> bytes, int start, int end) {
  final t = _table16;
  for (var i = start; i < end; i++) {
    c = t[((c >>> 8) ^ bytes[i]) & 0xFF] ^ ((c << 8) & 0xFFFF);
  }
  return c;
}

/// CRC-8/AUTOSAR of [bytes] (poly `0x2F`, init `0xFF`, xorout `0xFF`, no
/// reflection).
///
/// Only the low 8 bits of each element are used. The result is in
/// `0 .. 0xFF`: `crc8Autosar('123456789'.codeUnits) == 0xDF`,
/// `crc8Autosar([]) == 0`.
int crc8Autosar(List<int> bytes) =>
    _update8(0xFF, bytes, 0, bytes.length) ^ 0xFF;

/// CRC-16/IBM-3740, also called CRC-16/CCITT-FALSE, of [bytes] (poly
/// `0x1021`, init `0xFFFF`, xorout `0`, no reflection).
///
/// Only the low 8 bits of each element are used. The result is in
/// `0 .. 0xFFFF`: `crc16Ibm3740('123456789'.codeUnits) == 0x29B1`,
/// `crc16Ibm3740([]) == 0xFFFF`.
int crc16Ibm3740(List<int> bytes) => _update16(0xFFFF, bytes, 0, bytes.length);

/// Incremental CRC-8/AUTOSAR (same CRC as [crc8Autosar]); the input may be
/// split arbitrarily between [update] calls.
class Crc8Autosar {
  int _state = 0xFF;

  /// A CRC over the empty input.
  Crc8Autosar();

  /// Appends [bytes] (low 8 bits of each element) to the input.
  void update(List<int> bytes) {
    _state = _update8(_state, bytes, 0, bytes.length);
  }

  /// The CRC (`0 .. 0xFF`) of all bytes passed to [update] since creation or
  /// the last [reset]. Reading it does not change the state.
  int get value => _state ^ 0xFF;

  /// Same as [value].
  int finish() => value;

  /// Starts over with the empty input.
  void reset() {
    _state = 0xFF;
  }
}

/// Incremental CRC-16/IBM-3740 (same CRC as [crc16Ibm3740]); the input may be
/// split arbitrarily between [update] calls.
class Crc16Ibm3740 {
  int _state = 0xFFFF;

  /// A CRC over the empty input.
  Crc16Ibm3740();

  /// Appends [bytes] (low 8 bits of each element) to the input.
  void update(List<int> bytes) {
    _state = _update16(_state, bytes, 0, bytes.length);
  }

  /// The CRC (`0 .. 0xFFFF`) of all bytes passed to [update] since creation
  /// or the last [reset]. Reading it does not change the state.
  int get value => _state;

  /// Same as [value].
  int finish() => value;

  /// Starts over with the empty input.
  void reset() {
    _state = 0xFFFF;
  }
}

/// Length of the transmission CRC trailer for a frame of [n] bytes (header +
/// payload as on the wire, the CRC not counted): 1 (CRC-8/AUTOSAR) for
/// `n <= 14`, 2 (CRC-16/IBM-3740, little-endian: low byte first) for
/// `n >= 15` (carachangprotocol FRAMING.md section 2).
int transmissionCrcLen(int n) => n <= _crc8MaxN ? 1 : 2;

/// Transmission CRC of [frame] (header + payload, without the CRC):
/// [crc8Autosar] for `frame.length <= 14`, else [crc16Ibm3740].
int transmissionCrc(List<int> frame) =>
    frame.length <= _crc8MaxN ? crc8Autosar(frame) : crc16Ibm3740(frame);

/// A new list with [frame] (header + payload) followed by its transmission CRC
/// trailer (1 byte, or 2 bytes little-endian, see [transmissionCrcLen]).
Uint8List appendTransmissionCrc(List<int> frame) {
  final n = frame.length;
  final crc = transmissionCrc(frame);
  final out = Uint8List(n + transmissionCrcLen(n))..setRange(0, n, frame);
  if (n <= _crc8MaxN) {
    out[n] = crc;
  } else {
    out[n] = crc & 0xFF;
    out[n + 1] = crc >>> 8;
  }
  return out;
}

/// Checks the transmission CRC of a received frame (trailer included) and
/// returns the length of the frame without the trailer, or `null` if the
/// length is invalid (0, 1 or 16) or the CRC does not match. The frame itself is
/// `data.sublist(0, n)` (or `Uint8List.sublistView(data, 0, n)`). The CRC is
/// recomputed over the frame and compared with the trailer (CRC-16 read as a
/// little-endian 16-bit value).
int? stripTransmissionCrc(List<int> data) {
  final l = data.length;
  if (l <= 1 || l == _crc8MaxN + 2) return null;
  if (l <= _crc8MaxN + 1) {
    final n = l - 1;
    return (_update8(0xFF, data, 0, n) ^ 0xFF) == (data[n] & 0xFF) ? n : null;
  }
  final n = l - 2;
  final crc = _update16(0xFFFF, data, 0, n);
  return (crc & 0xFF) == (data[n] & 0xFF) && (crc >>> 8) == (data[n + 1] & 0xFF)
      ? n
      : null;
}

/// `true` if [data] (frame with trailer) has a valid length and a matching
/// transmission CRC, see [stripTransmissionCrc].
bool verifyTransmissionCrc(List<int> data) =>
    stripTransmissionCrc(data) != null;
