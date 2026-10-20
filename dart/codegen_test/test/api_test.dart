// Tests of the API of the generated code with hand-built values.
import 'dart:convert';
import 'dart:typed_data';

import 'package:bitis/bitis.dart';
import 'package:bitis_codegen_test/all_types.dart' as at;
import 'package:bitis_codegen_test/cara.dart';
import 'package:bitis_codegen_test/stress.dart' as stress;
import 'package:bitis_codegen_test/test_schema.dart' as ts;
import 'package:test/test.dart';

Object? jsonRoundTrip(Object? json) => jsonDecode(jsonEncode(json));

void main() {
  group('schema stamp', () {
    test('version and hash', () {
      expect(bitisVersion, matches(RegExp(r'^\d+\.\d+\.\d+')));
      expect(bitisSchemaHashHex, hasLength(16));
      expect(
        bitisSchemaHash.toHexString().toLowerCase().padLeft(16, '0'),
        bitisSchemaHashHex,
      );
      expect(ts.bitisSchemaHashHex, isNot(bitisSchemaHashHex));
    });
  });

  group('defaults (Rust Default semantics)', () {
    test('scalars, enums, oneofs, fixed point', () {
      const s = DTS_Msg_DeviceState();
      expect(s.wakeupReason, DTS_WakeupReason_Enum.coldStart);
      expect(s.epochAfterFoj2025, 0);
      expect(s.batterySoc, const FixedPoint.overflow());
      expect(s.isPluggedIn, isFalse);
      expect(DTS_Device_State.defaultValue, DTS_Device_State.off);
      expect(const DTS_Msg_CircuitryState().state, DTS_Device_State.off);
      expect(
        const STD_HandshakeResponse().value,
        isA<OO_StdHandshakeResponse_Value_Greeting>(),
      );
      expect(
        OO_DtsMsgWrapper_Value.defaultValue,
        const OO_DtsMsgWrapper_Value_Error(DTS_Error()),
      );
      expect(const DTS_Error().errMsg, '');
      expect(const DTS_Msg_WLanInfo_Entry().macElement, [0, 0, 0, 0, 0, 0]);
      expect(const STD_Msg().newNonce, isNull);
      expect(const STD_Msg().msgs, isEmpty);
    });

    test('64-bit types and binary (no const constructor)', () {
      final p = STS_Msg_PosGenData();
      expect(p.posContainerId, BigInt.zero);
      expect(p.height, isNull);
      expect(const STS_Msg_WLANPosReceived().posContainerId, Int64.ZERO);
      final l = ts.Lala();
      expect(l.lele, isEmpty);
      expect(l.ooLala, const ts.OO_Lala_OoLala_Test(0));
      expect(l.fixedPoint.isOverflow, isTrue);
      final a = at.AllTypes();
      expect(a.color, at.Color.green);
      expect(a.choice, const at.OO_AllTypes_Choice_Flag(false));
      expect(a.rfInner, [const at.Inner(), const at.Inner()]);
    });
  });

  group('hand-built messages', () {
    final gnss = DTS_Msg_GNSSPosition(
      tsAfterCps: 17,
      lat: const FixedPoint.value(52.520008),
      lng: const FixedPoint.value(13.404954),
      altitude: const FixedPoint.value(120),
      speed: const FixedPoint.value(3.5),
      direction: const FixedPoint.value(90),
      satsInView: 12,
      errLat: const FixedPoint.value(1.5),
      errLng: const FixedPoint.value(1.5),
      errAlt: const FixedPoint.underflow(),
    );
    final msg = DTS_Msg(
      msgs: [
        const DTS_MsgWrapper(
          value: OO_DtsMsgWrapper_Value_DeviceState(
            DTS_Msg_DeviceState(
              wakeupReason: DTS_WakeupReason_Enum.wakeupButton,
              epochAfterFoj2025: 1000,
              batterySoc: FixedPoint.value(80),
              isPluggedIn: true,
              chargingState: ChargerState_Enum.constantCurrent,
            ),
          ),
        ),
        DTS_MsgWrapper(value: OO_DtsMsgWrapper_Value_GnssPosition(gnss)),
        const DTS_MsgWrapper(
          value: OO_DtsMsgWrapper_Value_WlanInfos(
            DTS_Msg_WLanInfos(
              tsAfterCps: 3,
              macs: [
                DTS_Msg_WLanInfo_Entry(
                  macElement: [1, 2, 3, 4, 5, 6],
                  signalStrength: -80,
                ),
              ],
            ),
          ),
        ),
        const DTS_MsgWrapper(
          value: OO_DtsMsgWrapper_Value_StateInformation(
            DTS_Msg_StateInformation(
              value: OO_DtsMsgStateInformation_Value_BatteryState(
                DTS_Msg_BatteryState(
                  tsAfterAnchor: 5,
                  batterySoc: FixedPoint.value(42),
                  batteryVoltageTimestwo: FixedPoint.value(7.2),
                  chargingState: OO_DtsMsgBatteryState_ChargingState_PlugedIn(
                    DTS_Msg_PlugedIn(isCharging: true),
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
      nonce: 99,
    );

    test('byte round trip', () {
      final bytes = msg.toBytes();
      final back = DTS_Msg.fromBytesExact(bytes);
      expect(back, msg);
      expect(back.hashCode, msg.hashCode);
      expect(back.msgs[1].value, isA<OO_DtsMsgWrapper_Value_GnssPosition>());
    });

    test('pattern matching on oneofs is exhaustive', () {
      String describe(OO_DtsMsgWrapper_Value v) => switch (v) {
        OO_DtsMsgWrapper_Value_Error(:final value) => 'error ${value.errMsg}',
        OO_DtsMsgWrapper_Value_DeviceState(:final value) =>
          'state ${value.wakeupReason.bitisName}',
        OO_DtsMsgWrapper_Value_ClientPosStart() => 'start',
        OO_DtsMsgWrapper_Value_ClientPosEnd() => 'end',
        OO_DtsMsgWrapper_Value_WlanInfos(:final value) =>
          'wlan ${value.macs.length}',
        OO_DtsMsgWrapper_Value_GnssPosition(:final value) =>
          'gnss ${value.satsInView}',
        OO_DtsMsgWrapper_Value_Acknowledge() => 'ack',
        OO_DtsMsgWrapper_Value_StateInformation() => 'info',
      };
      expect(msg.msgs.map((m) => describe(m.value)), [
        'state WakeupButton',
        'gnss 12',
        'wlan 1',
        'info',
      ]);
    });

    test('fixed point equality is defined on the raw value', () {
      final a = gnss.copyWith(lat: const FixedPoint.value(52.52000800001));
      expect(a, gnss);
      expect(a.hashCode, gnss.hashCode);
      final b = gnss.copyWith(lat: const FixedPoint.value(-52.52));
      expect(b, isNot(gnss));
      // values above max are overflow on the wire
      expect(
        gnss.copyWith(speed: const FixedPoint.value(2000)),
        gnss.copyWith(speed: const FixedPoint.overflow()),
      );
      expect(DTS_Msg_GNSSPosition.latFpType, const FixedPointType(34, -90, 90));
    });

    test('lists are compared element by element', () {
      final a = DTS_Msg_WLanInfo_Entry(macElement: [1, 2, 3, 4, 5, 6]);
      final b = DTS_Msg_WLanInfo_Entry(macElement: List.of([1, 2, 3, 4, 5, 6]));
      expect(a, b);
      expect(a.hashCode, b.hashCode);
      expect(a, isNot(DTS_Msg_WLanInfo_Entry(macElement: [1, 2, 3, 4, 5, 7])));
      final l1 = ts.Lala(lele: Uint8List.fromList([1, 2]));
      final l2 = ts.Lala(lele: Uint8List.fromList([1, 2]));
      expect(l1, l2);
      expect(l1.hashCode, l2.hashCode);
    });

    test('copyWith keeps, replaces and clears', () {
      const m = STD_Msg(newNonce: 5);
      expect(m.copyWith(), m);
      expect(m.copyWith(newNonce: 6).newNonce, 6);
      expect(m.copyWith(newNonce: null).newNonce, isNull);
      expect(m.copyWith(msgs: [const STD_MsgWrapper()]).newNonce, 5);
      expect(() => m.copyWith(newNonce: 'x'), throwsA(isA<TypeError>()));
    });

    test('toString', () {
      expect(const DTS_Msg_Ping(dummy: 2).toString(), 'DTS_Msg_Ping(dummy: 2)');
      expect(
        const OO_StdHandshakeResponse_Value_Err(
          STD_HandshakeError_Enum.timeOut,
        ).toString(),
        'OO_StdHandshakeResponse_Value_Err(STD_HandshakeError_Enum.timeOut)',
      );
    });
  });

  group('64-bit values', () {
    test('uint_64 as BigInt, int_64 as Int64, JSON strings beyond 2^53', () {
      final p = STS_Msg_PosGenData(
        posContainerId: BigInt.parse('18446744073709551615'),
        height: -5,
        source: STS_PosSource_Enum.wlan,
      );
      final back = STS_Msg_PosGenData.fromBytes(p.toBytes());
      expect(back.posContainerId, BigInt.parse('18446744073709551615'));
      final json = p.toJson();
      expect(json['pos_container_id'], '18446744073709551615');
      expect(json['source'], 'wlan');
      expect(json['height'], -5);
      expect(json.containsKey('speed'), isFalse);
      expect(STS_Msg_PosGenData.fromJson(json), p);

      final w = STS_Msg_WLANPosReceived(
        posContainerId: Int64.MIN_VALUE + 1,
        tsEpochAfterFoj2025Container: Int64(123),
      );
      final wj = w.toJson();
      expect(wj['pos_container_id'], '-9223372036854775807');
      expect(wj['ts_epoch_after_foj2025_container'], 123);
      expect(STS_Msg_WLANPosReceived.fromBytes(w.toBytes()), w);
    });

    test('out of range values are rejected when encoding', () {
      expect(
        () =>
            STS_Msg_WLANPosReceived(posContainerId: Int64.MIN_VALUE).toBytes(),
        throwsA(isA<BitisEncodeException>()),
      );
      expect(
        () => STS_Msg_PosGenData(posContainerId: BigInt.two.pow(64)).toBytes(),
        throwsA(isA<BitisEncodeException>()),
      );
      expect(
        () => const DTS_Msg_Ping(dummy: 4).toBytes(),
        throwsA(isA<BitisEncodeException>()),
      );
      expect(
        () => const DTS_Msg_WLanInfo_Entry(signalStrength: -256).toBytes(),
        throwsA(isA<BitisEncodeException>()),
      );
      // int_8 is sign-magnitude: -255..255 are valid
      expect(
        DTS_Msg_WLanInfo_Entry.fromBytes(
          const DTS_Msg_WLanInfo_Entry(signalStrength: -255).toBytes(),
        ).signalStrength,
        -255,
      );
      // int_16: the base type range rule (spec 2.3) excludes -32768
      expect(
        () => STS_Msg_PosGenData(speed: -32768).toBytes(),
        throwsA(isA<BitisEncodeException>()),
      );
      expect(
        () => const DTS_Msg_WLanInfo_Entry(macElement: [1, 2, 3]).toBytes(),
        throwsA(isA<BitisEncodeException>()),
      );
      expect(
        () => const DTS_Error(errMsg: 'grüß').toBytes(),
        throwsA(isA<BitisEncodeException>()),
      );
    });
  });

  group('JSON mapping', () {
    test('enums, oneofs, optionals, fixed point, binary', () {
      final l = ts.Lala(
        lili: [true],
        lolo: 3,
        lele: Uint8List.fromList([0, 255]),
        name: 'x',
        fixedPoint: const FixedPoint.underflow(),
        pipi: const ts.Pipi(num: ts.Numbers.three),
        ooLala: const ts.OO_Lala_OoLala_Papa(
          ts.Papa(ooLala: ts.OO_Papa_OoLala_Test2(0.5)),
        ),
      );
      final j = jsonRoundTrip(l.toJson())! as Map<String, Object?>;
      expect(j['lolo'], 3);
      expect(j['lele'], 'AP8=');
      expect(j['fixed_point'], {'state': 'underflow'});
      expect(j['pipi'], {'num': 'Three'});
      expect(j['nah'], 'TwoAndAHalf');
      expect(j['oo_lala'], {
        'papa': {
          'oo_lala': {'test2': 0.5},
        },
      });
      expect(ts.Lala.fromJson(j), l);
      // missing optional and null are the same
      final j2 = Map.of(j)..remove('lolo');
      expect(ts.Lala.fromJson(j2).lolo, isNull);
      expect(ts.Lala.fromJson(Map.of(j)..['lolo'] = null).lolo, isNull);
      expect(DTS_Device_Circuitry.fromJson('CPU'), DTS_Device_Circuitry.cpu);
      expect(DTS_Device_Circuitry.cpu.toJson(), 'CPU');
      expect(
        const DTS_Msg_CircuitryState(
          circuitry: DTS_Device_Circuitry.wlan,
        ).toJson(),
        {'ts_after_anchor': 0, 'circuitry': 'WLAN', 'state': 'Off'},
      );
    });

    test('invalid JSON is a FormatException', () {
      final j = const DTS_Msg_CircuitryState().toJson();
      expect(
        () => DTS_Msg_CircuitryState.fromJson({...j, 'circuitry': 'cpu'}),
        throwsFormatException,
      );
      expect(
        () => DTS_Msg_CircuitryState.fromJson({...j, 'ts_after_anchor': '1x'}),
        throwsFormatException,
      );
      expect(
        () => DTS_Msg_CircuitryState.fromJson({...j}..remove('state')),
        throwsFormatException,
      );
      expect(
        () => OO_DtsMsgWrapper_Value.fromJson({'nope': <String, Object?>{}}),
        throwsFormatException,
      );
      expect(
        () => OO_DtsMsgWrapper_Value.fromJson({'ack': 1, 'error': 2}),
        throwsFormatException,
      );
      expect(
        () => DTS_Error.fromJson({'err': 'UnspecifiedError', 'err_msg': 'ä'}),
        throwsFormatException,
      );
    });
  });

  group('decoding errors', () {
    test('unknown oneof index and enum index', () {
      // DTS_MsgWrapper: oneof(4) with 8 elements; index 9 = 1 1001 0 ...
      final w = BitWriter()..writeDynUint(9, 32, 4);
      expect(
        () => DTS_MsgWrapper.fromBytes(w.toBytes()),
        throwsA(
          isA<BitisDecodeException>().having(
            (e) => e.kind,
            'kind',
            BitisDecodeErrorKind.invalidDiscriminant,
          ),
        ),
      );
      // DTS_Msg_CircuitryState: uint_9, enum(2) with 4 values -> index 4
      final e =
          BitWriter()
            ..writeUint(0, 9)
            ..writeDynUint(4, 32, 2);
      expect(
        () => DTS_Msg_CircuitryState.fromBytes(e.toBytes()),
        throwsA(
          isA<BitisDecodeException>().having(
            (e) => e.kind,
            'kind',
            BitisDecodeErrorKind.invalidDiscriminant,
          ),
        ),
      );
    });
  });

  group('names', () {
    test('attributes named like locals of the generated methods', () {
      const s = stress.Shadowing(
        w: 1,
        other: 2,
        value: 3,
        json: 4,
        r: 5,
        v: 6,
        data: 7,
        optW: 1,
      );
      expect(stress.Shadowing.fromBytes(s.toBytes()), s);
      expect(stress.Shadowing.fromJson(s.toJson()), s);
      expect(s.copyWith(w: 7).w, 7);
      expect(s, isNot(s.copyWith(other: 3)));
    });

    test('long names, non-const defaults', () {
      final m = stress.UsesNonConst();
      expect(stress.UsesNonConst.fromBytes(m.toBytes()), m);
      expect(
        m.m.aOneofWithALongNameToTestTheLayout,
        isA<
          stress.OO_AMessageWithAReallyReallyLongNameThatDoesNotFit_AOneofWithALongNameToTestTheLayout_BinaryDefault
        >(),
      );
    });
  });
}
