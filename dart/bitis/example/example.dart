// Hand-written equivalent of what `bitis -l dart` generates for
//   msg Reading { uint_12 id; fp_10[-40, 125] temp; optional astr_d4 name; }
import 'package:bitis/bitis.dart';

/// Example message.
class Reading extends BitisMessage {
  /// Creates a reading.
  Reading(this.id, this.temp, this.name);

  /// uint_12
  int id;

  /// fp_10[-40, 125]
  FixedPoint temp;

  /// optional astr_d4
  String? name;

  static const _tempType = FixedPointType(10, -40, 125);

  @override
  void bitisWrite(BitWriter w) {
    w.writeUint(id, 12);
    w.writeFixedPoint(temp, _tempType);
    w.writeOptional(name, (w, v) => w.writeAStr(v, 4));
  }

  /// Reads a reading.
  static Reading bitisRead(BitReader r) => Reading(
        r.readUint(12),
        r.readFixedPoint(_tempType),
        r.readOptional((r) => r.readAStr(4)),
      );
}

void main() {
  final bytes = bitisEncode(Reading(7, const FixedPoint.value(21.5), 'esp'));
  final decoded = bitisDecode(bytes, Reading.bitisRead);
  print('${bytes.length} bytes, ${decoded.totalBits} bits: '
      'id=${decoded.value.id} temp=${decoded.value.temp} '
      'name=${decoded.value.name}');
  final strict = bitisDecodeExact(bytes, Reading.bitisRead);
  print(strict.temp.value);
}
