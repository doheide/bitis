// Manifest parser and dispatch by message type name (shared by the tests).
//
// The manifest format is described in golden/README.md. The manifest is
// embedded as a string (manifest.g.dart) so that the same tests run on the VM
// and in the browser.
import 'dart:convert';
import 'dart:typed_data';

import 'package:bitis/bitis.dart';

import '../../gen/dart/golden.dart';
import 'manifest.g.dart';

/// Kind of a manifest record.
enum RecordKind {
  /// `valid <case> <MsgType> <total_bits> <hex>`
  valid,

  /// `noncanon <case> <MsgType> <total_bits> <input_hex> <canonical_hex>`
  noncanon,

  /// `invalid <case> <MsgType> <error> <default|exact> <hex>`
  invalid,

  /// `min_bits <MsgType> <min_bits>`
  minBits,

  /// `max_bits <MsgType> <max_bits|unbounded>`
  maxBits,
}

/// One line of the manifest.
final class Record {
  /// Creates a record.
  Record(
    this.kind,
    this.name,
    this.type, {
    this.bits = 0,
    this.maxBits,
    Uint8List? data,
    Uint8List? canonical,
    this.error,
    this.exact = false,
  }) : data = data ?? Uint8List(0),
       canonical = canonical ?? Uint8List(0);

  /// Record kind.
  final RecordKind kind;

  /// Case name (`min_bits`: the message type).
  final String name;

  /// Message type name.
  final String type;

  /// valid/noncanon: total_bits, min_bits: min_bits.
  final int bits;

  /// max_bits: max_bits, `null` = unbounded.
  final int? maxBits;

  /// valid: the bytes, noncanon: the input, invalid: the input.
  final Uint8List data;

  /// noncanon: the canonical re-encoding.
  final Uint8List canonical;

  /// invalid: the expected error.
  final BitisDecodeErrorKind? error;

  /// invalid: decode with the exact decoder (`bitisDecodeExact`).
  final bool exact;
}

/// Lower case hex without separators, `-` for no bytes (as in the manifest).
String hex(List<int> d) =>
    d.isEmpty ? '-' : d.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

/// Inverse of [hex].
Uint8List unhex(String s) {
  if (s == '-') return Uint8List(0);
  if (s.length.isOdd || !RegExp(r'^[0-9a-f]+$').hasMatch(s)) {
    throw FormatException('bad hex string', s);
  }
  return Uint8List.fromList([
    for (var i = 0; i < s.length; i += 2)
      int.parse(s.substring(i, i + 2), radix: 16),
  ]);
}

/// Manifest name of an error kind: the Dart enum name with an upper case first
/// letter (`unexpectedEnd` -> `UnexpectedEnd`), the names of spec section 3.
String manifestErrorName(BitisDecodeErrorKind kind) =>
    kind.name[0].toUpperCase() + kind.name.substring(1);

/// The error kind with the manifest name [name].
BitisDecodeErrorKind parseErrorName(String name) {
  for (final k in BitisDecodeErrorKind.values) {
    if (manifestErrorName(k) == name) return k;
  }
  throw FormatException('unknown error kind', name);
}

List<Record> _parse(String text) {
  final recs = <Record>[];
  final lines = const LineSplitter().convert(text);
  for (var n = 0; n < lines.length; n++) {
    final line = lines[n].trim();
    if (line.isEmpty || line.startsWith('#')) continue;
    final f = line.split(' ');
    Never malformed() =>
        throw FormatException('manifest line ${n + 1}: malformed', line);
    int number(String s) => int.tryParse(s) ?? malformed();
    switch ((f[0], f.length)) {
      case ('valid', 5):
        recs.add(
          Record(
            RecordKind.valid,
            f[1],
            f[2],
            bits: number(f[3]),
            data: unhex(f[4]),
          ),
        );
      case ('noncanon', 6):
        recs.add(
          Record(
            RecordKind.noncanon,
            f[1],
            f[2],
            bits: number(f[3]),
            data: unhex(f[4]),
            canonical: unhex(f[5]),
          ),
        );
      case ('invalid', 6):
        recs.add(
          Record(
            RecordKind.invalid,
            f[1],
            f[2],
            error: parseErrorName(f[3]),
            exact: switch (f[4]) {
              'exact' => true,
              'default' => false,
              _ => malformed(),
            },
            data: unhex(f[5]),
          ),
        );
      case ('min_bits', 3):
        recs.add(Record(RecordKind.minBits, f[1], f[1], bits: number(f[2])));
      case ('max_bits', 3):
        recs.add(
          Record(
            RecordKind.maxBits,
            'max_bits_${f[1]}',
            f[1],
            maxBits: f[2] == 'unbounded' ? null : number(f[2]),
          ),
        );
      default:
        malformed();
    }
  }
  return recs;
}

/// All records of the manifest.
final List<Record> manifest = _parse(goldenManifest);

/// The records of [kind].
List<Record> records(RecordKind kind) =>
    manifest.where((r) => r.kind == kind).toList();

/// The bytes of the valid or non-canonical case [name].
Uint8List caseBytes(String name) =>
    manifest
        .firstWhere(
          (r) =>
              (r.kind == RecordKind.valid || r.kind == RecordKind.noncanon) &&
              r.name == name,
          orElse: () => throw ArgumentError.value(name, 'name', 'no such case'),
        )
        .data;

/// Result of decoding and re-encoding.
final class Roundtrip {
  /// Creates a result.
  const Roundtrip(this.value, this.bits, this.reencoded, this.reencodedBits);

  /// The decoded message.
  final BitisMessage value;

  /// Bits consumed by the decoder.
  final int bits;

  /// Re-encoding of [value].
  final Uint8List reencoded;

  /// Significant bits of [reencoded].
  final int reencodedBits;
}

/// The generated API of one message class.
final class MessageType<T extends BitisMessage> {
  /// Creates the entry.
  const MessageType(
    this.name,
    this.read,
    this.fromBytesExact,
    this.minBits,
    this.maxBits,
    this.fromJson,
    this.toJson,
  );

  /// Message name in the schema.
  final String name;

  /// `T.bitisRead`.
  final BitisReadFn<T> read;

  /// `T.fromBytesExact`.
  final T Function(List<int> data) fromBytesExact;

  /// `T.bitisMinBits`.
  final int minBits;

  /// `T.bitisMaxBits` (`null` = unbounded).
  final int? maxBits;

  /// `T.fromJson`.
  final T Function(Map<String, dynamic> json) fromJson;

  /// `T.toJson` of a message.
  final Map<String, Object?> Function(T message) toJson;

  /// Decodes [data] (default decoder, or exact decoder if [exact]) and
  /// re-encodes the value. Throws [BitisDecodeException] for invalid input.
  Roundtrip roundtrip(List<int> data, {required bool exact}) {
    final T value;
    final int bits;
    if (exact) {
      value = fromBytesExact(data);
      bits = bitisDecode(data, read).totalBits;
    } else {
      final d = bitisDecode(data, read);
      value = d.value;
      bits = d.totalBits;
    }
    // re-encoding a decoded value must not fail
    final enc = bitisEncodeWithSize(value);
    return Roundtrip(value, bits, enc.bytes, enc.totalBits);
  }

  /// Decodes [data] exactly, converts the value to a JSON string and back
  /// and returns the encoding of the result (the app exchanges JSON).
  Uint8List jsonRoundtrip(List<int> data) {
    final json = jsonEncode(toJson(fromBytesExact(data)));
    final decoded = jsonDecode(json) as Map<String, dynamic>;
    return bitisEncode(fromJson(decoded));
  }
}

/// The generated message class with the schema name [name] (all 16 message
/// types of golden/schema/golden.bitis).
MessageType<BitisMessage> messageType(String name) => switch (name) {
  'Empty' => MessageType<Empty>(
    name,
    Empty.bitisRead,
    Empty.fromBytesExact,
    Empty.bitisMinBits,
    Empty.bitisMaxBits,
    Empty.fromJson,
    (m) => m.toJson(),
  ),
  'Ints' => MessageType<Ints>(
    name,
    Ints.bitisRead,
    Ints.fromBytesExact,
    Ints.bitisMinBits,
    Ints.bitisMaxBits,
    Ints.fromJson,
    (m) => m.toJson(),
  ),
  'DynInts' => MessageType<DynInts>(
    name,
    DynInts.bitisRead,
    DynInts.fromBytesExact,
    DynInts.bitisMinBits,
    DynInts.bitisMaxBits,
    DynInts.fromJson,
    (m) => m.toJson(),
  ),
  'Floats' => MessageType<Floats>(
    name,
    Floats.bitisRead,
    Floats.fromBytesExact,
    Floats.bitisMinBits,
    Floats.bitisMaxBits,
    Floats.fromJson,
    (m) => m.toJson(),
  ),
  'FixedPoints' => MessageType<FixedPoints>(
    name,
    FixedPoints.bitisRead,
    FixedPoints.fromBytesExact,
    FixedPoints.bitisMinBits,
    FixedPoints.bitisMaxBits,
    FixedPoints.fromJson,
    (m) => m.toJson(),
  ),
  'Texts' => MessageType<Texts>(
    name,
    Texts.bitisRead,
    Texts.fromBytesExact,
    Texts.bitisMinBits,
    Texts.bitisMaxBits,
    Texts.fromJson,
    (m) => m.toJson(),
  ),
  'Leaf' => MessageType<Leaf>(
    name,
    Leaf.bitisRead,
    Leaf.fromBytesExact,
    Leaf.bitisMinBits,
    Leaf.bitisMaxBits,
    Leaf.fromJson,
    (m) => m.toJson(),
  ),
  'Mid' => MessageType<Mid>(
    name,
    Mid.bitisRead,
    Mid.fromBytesExact,
    Mid.bitisMinBits,
    Mid.bitisMaxBits,
    Mid.fromJson,
    (m) => m.toJson(),
  ),
  'Nested' => MessageType<Nested>(
    name,
    Nested.bitisRead,
    Nested.fromBytesExact,
    Nested.bitisMinBits,
    Nested.bitisMaxBits,
    Nested.fromJson,
    (m) => m.toJson(),
  ),
  'Containers' => MessageType<Containers>(
    name,
    Containers.bitisRead,
    Containers.fromBytesExact,
    Containers.bitisMinBits,
    Containers.bitisMaxBits,
    Containers.fromJson,
    (m) => m.toJson(),
  ),
  'Enums' => MessageType<Enums>(
    name,
    Enums.bitisRead,
    Enums.fromBytesExact,
    Enums.bitisMinBits,
    Enums.bitisMaxBits,
    Enums.fromJson,
    (m) => m.toJson(),
  ),
  'IntList' => MessageType<IntList>(
    name,
    IntList.bitisRead,
    IntList.fromBytesExact,
    IntList.bitisMinBits,
    IntList.bitisMaxBits,
    IntList.fromJson,
    (m) => m.toJson(),
  ),
  'Choice' => MessageType<Choice>(
    name,
    Choice.bitisRead,
    Choice.fromBytesExact,
    Choice.bitisMinBits,
    Choice.bitisMaxBits,
    Choice.fromJson,
    (m) => m.toJson(),
  ),
  'Everything' => MessageType<Everything>(
    name,
    Everything.bitisRead,
    Everything.fromBytesExact,
    Everything.bitisMinBits,
    Everything.bitisMaxBits,
    Everything.fromJson,
    (m) => m.toJson(),
  ),
  'BoundsItem' => MessageType<BoundsItem>(
    name,
    BoundsItem.bitisRead,
    BoundsItem.fromBytesExact,
    BoundsItem.bitisMinBits,
    BoundsItem.bitisMaxBits,
    BoundsItem.fromJson,
    (m) => m.toJson(),
  ),
  'Bounds' => MessageType<Bounds>(
    name,
    Bounds.bitisRead,
    Bounds.fromBytesExact,
    Bounds.bitisMinBits,
    Bounds.bitisMaxBits,
    Bounds.fromJson,
    (m) => m.toJson(),
  ),
  _ => throw ArgumentError.value(name, 'name', 'unknown message type'),
};

/// Decodes the valid or non-canonical case [name] with the exact decoder.
T decodeCase<T extends BitisMessage>(String name, BitisReadFn<T> read) =>
    bitisDecodeExact(caseBytes(name), read);
