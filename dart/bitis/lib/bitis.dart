/// Pure-Dart runtime for the bitis bit-level serialization format.
///
/// Web-safe (dart2js and dart2wasm): no `dart:io`, `dart:ffi` or
/// `dart:mirrors`; 64-bit values are handled as two 32-bit halves.
///
/// Main entry points:
/// * [BitWriter] / [BitReader]: one `write*` / `read*` method per bitis type,
/// * [BitisMessage], [bitisEncode], [bitisDecode], [bitisDecodeExact],
/// * [FixedPoint] / [FixedPointType] for `fp_N[min, max]`,
/// * [BitisDecodeException] / [BitisEncodeException],
/// * [MinBits], [MaxBits] (spec section 4) and [BitisJson] (JSON mapping
///   helpers),
/// * [BitisEq], [bitisKeep], [bitisKeepOr] (used by generated code),
/// * [crc32] / [Crc32]: CRC-32 helper (not part of the wire format).
///
/// The wire format is specified in `docs/WIRE_FORMAT.md` of the bitis
/// repository.
library;

export 'package:fixnum/fixnum.dart' show Int64;

export 'src/bit_reader.dart';
export 'src/bit_writer.dart';
export 'src/codec.dart';
export 'src/crc32.dart';
export 'src/errors.dart';
export 'src/fixed_point.dart' show FixedPoint, FixedPointState, FixedPointType;
export 'src/json.dart';
export 'src/max_bits.dart';
export 'src/min_bits.dart';
export 'src/support.dart';
