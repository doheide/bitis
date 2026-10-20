import 'package:bitis/bitis.dart';

/// Operations of a generated message class (see messages.g.dart).
class MessageOps {
  /// Creates the operations.
  const MessageOps(
    this.create,
    this.fromJson,
    this.fromBytes,
    this.fromBytesExact,
    this.minBits,
    this.maxBits,
    this.toJson,
  );

  /// Default constructor (all attributes default).
  final BitisMessage Function() create;

  /// `fromJson` factory.
  final BitisMessage Function(Map<String, dynamic> json) fromJson;

  /// `fromBytes`.
  final BitisMessage Function(List<int> data) fromBytes;

  /// `fromBytesExact`.
  final BitisMessage Function(List<int> data) fromBytesExact;

  /// `bitisMinBits`.
  final int minBits;

  /// `bitisMaxBits` (null: unbounded).
  final int? maxBits;

  /// `toJson` of a message of this type.
  final Map<String, Object?> Function(BitisMessage message) toJson;
}
