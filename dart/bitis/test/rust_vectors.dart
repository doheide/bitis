// GENERATED from the Rust reference runtime by dart/bitis/tool/rust_vectors
// (see its Cargo.toml for how to regenerate). Do not edit. Only valid values.
//
// Item specs: bool, u<X>, i<X>, du<X>d<Y>, di<X>d<Y>, f32, f64 (values are
// IEEE bit patterns in hex), fp<N>[min,max] (value 'input>decoded', U/O for
// underflow/overflow, else f64 bits in hex), astr<Y>, bin<Y> (hex),
// opt<T>, fa<N><T>, da<Y><T>.

// ignore_for_file: lines_longer_than_80_chars

/// One sequence of values serialized into one bit stream.
class RustVector {
  /// Creates a vector.
  const RustVector(this.name, this.items, this.hex, this.bits);

  /// Name.
  final String name;

  /// (type spec, value) pairs in stream order.
  final List<(String, Object?)> items;

  /// Encoded bytes as hex.
  final String hex;

  /// Number of significant bits.
  final int bits;
}

/// All vectors.
const rustVectors = <RustVector>[
  RustVector(
    'bool',
    [('bool', 'true'), ('bool', 'false'), ('bool', 'true')],
    '05',
    3,
  ),
  RustVector(
    'uint_1',
    [('u1', '0'), ('u1', '1')],
    '02',
    2,
  ),
  RustVector(
    'uint_5',
    [('u5', '0'), ('u5', '1'), ('u5', '17'), ('u5', '31')],
    '20c40f',
    20,
  ),
  RustVector(
    'uint_8',
    [('u8', '255'), ('u8', '128')],
    'ff80',
    16,
  ),
  RustVector(
    'uint_16',
    [('u16', '65535'), ('u16', '256')],
    'ffff0001',
    32,
  ),
  RustVector(
    'uint_17',
    [('u17', '131071'), ('u17', '65536')],
    'ffff010002',
    34,
  ),
  RustVector(
    'uint_31',
    [('u31', '2147483647')],
    'ffffff7f',
    31,
  ),
  RustVector(
    'uint_32',
    [('u32', '4294967295'), ('u32', '2147483648'), ('u32', '305419896')],
    'ffffffff0000008078563412',
    96,
  ),
  RustVector(
    'uint_33',
    [('u33', '8589934591'), ('u33', '4294967296')],
    'ffffffff0100000002',
    66,
  ),
  RustVector(
    'uint_53',
    [('u53', '9007199254740991'), ('u53', '320255973501901')],
    'ffffffffffffbf7935f1ac682400',
    106,
  ),
  RustVector(
    'uint_63',
    [('u63', '9223372036854775807'), ('u63', '4611686018427387904')],
    'ffffffffffffff7f0000000000000020',
    126,
  ),
  RustVector(
    'uint_64',
    [
      ('u64', '18446744073709551615'),
      ('u64', '9223372036854775808'),
      ('u64', '0'),
      ('u64', '81985529216486895')
    ],
    'ffffffffffffffff00000000000000800000000000000000efcdab8967452301',
    256,
  ),
  RustVector(
    'int_1',
    [('i1', '-1'), ('i1', '0'), ('i1', '1')],
    '23',
    6,
  ),
  RustVector(
    'int_5',
    [('i5', '-31'), ('i5', '31'), ('i5', '-1'), ('i5', '0')],
    'bf3f00',
    24,
  ),
  RustVector(
    'int_15',
    [('i15', '-32767'), ('i15', '32767')],
    'fffffeff',
    32,
  ),
  RustVector(
    'int_16',
    [('i16', '-32767'), ('i16', '32767'), ('i16', '-256')],
    'fffffcff050800',
    51,
  ),
  RustVector(
    'int_17',
    [('i17', '-131071'), ('i17', '131071')],
    'fffffbff0f',
    36,
  ),
  RustVector(
    'int_32',
    [('i32', '-2147483647'), ('i32', '2147483647'), ('i32', '-5')],
    'fffffffffcffffff2d00000000',
    99,
  ),
  RustVector(
    'int_33',
    [('i33', '-8589934591'), ('i33', '8589934591')],
    'fffffffffbffffff0f',
    68,
  ),
  RustVector(
    'int_53',
    [('i53', '-9007199254740991'), ('i53', '4503599627370496')],
    'ffffffffffff3f00000000000008',
    108,
  ),
  RustVector(
    'int_63',
    [('i63', '-9223372036854775807'), ('i63', '9223372036854775807')],
    'fffffffffffffffffeffffffffffffff',
    128,
  ),
  RustVector(
    'int_64',
    [
      ('i64', '-9223372036854775807'),
      ('i64', '9223372036854775807'),
      ('i64', '-1')
    ],
    'fffffffffffffffffcffffffffffffff0d0000000000000000',
    195,
  ),
  RustVector(
    'uint_32d3',
    [
      ('du32d3', '0'),
      ('du32d3', '10'),
      ('du32d3', '7'),
      ('du32d3', '8'),
      ('du32d3', '4294967295')
    ],
    '6abc18ffffffffff07',
    67,
  ),
  RustVector(
    'uint_8d8',
    [('du8d8', '0'), ('du8d8', '255')],
    'fe03',
    10,
  ),
  RustVector(
    'uint_16d5',
    [
      ('du16d5', '65535'),
      ('du16d5', '32768'),
      ('du16d5', '31'),
      ('du16d5', '32')
    ],
    'ffff1f04c1bf6000',
    60,
  ),
  RustVector(
    'uint_1d1',
    [('du1d1', '0'), ('du1d1', '1')],
    '06',
    3,
  ),
  RustVector(
    'uint_32d32',
    [('du32d32', '4294967295'), ('du32d32', '1')],
    'ffffffff0700000000',
    66,
  ),
  RustVector(
    'uint_40d6',
    [('du40d6', '1099511627775'), ('du40d6', '68719476736')],
    'ffffffffffff402010080406',
    94,
  ),
  RustVector(
    'uint_63d7',
    [('du63d7', '9223372036854775807'), ('du63d7', '128')],
    'ffffffffffffffffff010300',
    89,
  ),
  RustVector(
    'uint_64d8',
    [
      ('du64d8', '18446744073709551615'),
      ('du64d8', '9223372036854775808'),
      ('du64d8', '0')
    ],
    'ffffffffffffffffff01020408102040808000',
    145,
  ),
  RustVector(
    'uint_64d33',
    [('du64d33', '18446744073709551615'), ('du64d33', '8589934592')],
    'ffffffffffffffff070000003000000000',
    132,
  ),
  RustVector(
    'int_32d3',
    [
      ('di32d3', '0'),
      ('di32d3', '-10'),
      ('di32d3', '10'),
      ('di32d3', '2147483647'),
      ('di32d3', '-2147483647')
    ],
    'aca186fffffffffffdffffffff1f',
    110,
  ),
  RustVector(
    'int_16d4',
    [('di16d4', '32767'), ('di16d4', '-32767')],
    'feffefffff01',
    42,
  ),
  RustVector(
    'int_8d3',
    [('di8d3', '255'), ('di8d3', '-255'), ('di8d3', '-1')],
    'feffff07',
    30,
  ),
  RustVector(
    'int_64d8',
    [
      ('di64d8', '9223372036854775807'),
      ('di64d8', '-9223372036854775807'),
      ('di64d8', '0')
    ],
    'fefffffffffffffffffeffffffffffffffff01',
    148,
  ),
  RustVector(
    'int_48d10',
    [('di48d10', '281474976710655'), ('di48d10', '-1099511627776')],
    'feffffffffffff00042000011800',
    108,
  ),
  RustVector(
    'float',
    [
      ('f32', '00000000'),
      ('f32', '80000000'),
      ('f32', '3fc00000'),
      ('f32', 'c0100000'),
      ('f32', '7fc00000'),
      ('f32', '7f800000'),
      ('f32', 'ff800000'),
      ('f32', '00800000'),
      ('f32', '00000001'),
      ('f32', '7f7fffff'),
      ('f32', '3dcccccd')
    ],
    '00000000000000800000c03f000010c00000c07f0000807f000080ff0000800001000000ffff7f7fcdcccc3d',
    352,
  ),
  RustVector(
    'double',
    [
      ('f64', '0000000000000000'),
      ('f64', '8000000000000000'),
      ('f64', '400921fb54442d18'),
      ('f64', '7ff8000000000000'),
      ('f64', '7ff0000000000000'),
      ('f64', 'fff0000000000000'),
      ('f64', '0000000000000001'),
      ('f64', '7fefffffffffffff'),
      ('f64', 'fe37e43c8800759c'),
      ('f64', '3fb999999999999a')
    ],
    '00000000000000000000000000000080182d4454fb210940000000000000f87f000000000000f07f000000000000f0ff0100000000000000ffffffffffffef7f9c7500883ce437fe9a9999999999b93f',
    640,
  ),
  RustVector(
    'fp10[-5,100]',
    [
      ('fp10[-5,100]', 'c014000000000000>c014000000000000'),
      ('fp10[-5,100]', '4059000000000000>4059000000000000'),
      ('fp10[-5,100]', '0000000000000000>3fa40f0b48765900'),
      ('fp10[-5,100]', '4047c00000000000>4047c694efb3c6d5'),
      ('fp10[-5,100]', '4028b0a3d70a3d71>4028c291ed721590'),
      ('fp10[-5,100]', 'U>U'),
      ('fp10[-5,100]', 'O>O')
    ],
    '01f82f0380aa00f03f',
    70,
  ),
  RustVector(
    'fp8[0,253] ties',
    [
      ('fp8[0,253]', '3fe0000000000000>3fefffffffffffff'),
      ('fp8[0,253]', '3ff8000000000000>3fffffffffffffff'),
      ('fp8[0,253]', '4004000000000000>4008000000000000'),
      ('fp8[0,253]', '406f900000000000>406fa00000000000'),
      ('fp8[0,253]', '3fdfffffffffffff>3fefffffffffffff'),
      ('fp8[0,253]', '8000000000000000>0000000000000000'),
      ('fp8[0,253]', '406fa00000000000>406fa00000000000')
    ],
    '020304fe0201fe',
    56,
  ),
  RustVector(
    'fp2[0,1]',
    [
      ('fp2[0,1]', '0000000000000000>0000000000000000'),
      ('fp2[0,1]', '3ff0000000000000>3ff0000000000000'),
      ('fp2[0,1]', '3fe0000000000000>3ff0000000000000'),
      ('fp2[0,1]', '3fdffe5c91d14e3c>0000000000000000')
    ],
    '69',
    8,
  ),
  RustVector(
    'fp16[-40,125]',
    [
      ('fp16[-40,125]', '40355eb851eb851f>40355e861b9252b8'),
      ('fp16[-40,125]', 'c043ffdf3b645a1d>c044000000000000'),
      ('fp16[-40,125]', '405f3ffe5c91d14e>405f400000000000')
    ],
    '375f0100feff',
    48,
  ),
  RustVector(
    'fp53[-1000000,1000000]',
    [
      ('fp53[-1000000,1000000]', '3fb999999999999a>3fb999999a000000'),
      ('fp53[-1000000,1000000]', 'c12e847e3f35ba6e>c12e847e3f35ba6d'),
      ('fp53[-1000000,1000000]', '412e848000000000>412e848000000000'),
      ('fp53[-1000000,1000000]', 'O>O')
    ],
    '9af2d71a000010e872691d0000f8ffffffffffffffffffffffff0f',
    212,
  ),
  RustVector(
    'fp33[-5000000000,5000000000]',
    [
      ('fp33[-5000000000,5000000000]', '419d6f3456000000>419d6f34555c5500'),
      ('fp33[-5000000000,5000000000]', 'c1f2a05f20000000>c1f2a05f20000000')
    ],
    '3e2c52060300000000',
    66,
  ),
  RustVector(
    'astr',
    [
      ('astr4', ''),
      ('astr4', 'hello'),
      ('astr3', '\x00\x7f!\x27\x24\x5c'),
      (
        'astr8',
        '0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz'
      )
    ],
    '16b43236b6b706f0177242c2956800981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d981899199a1a9b1b9c9c30b131b232b333b434b535b636b737b838b939ba3abb3bbc3c3d',
    2711,
  ),
  RustVector(
    'binary',
    [
      ('bin3', ''),
      ('bin3', '00ff0180'),
      (
        'bin8',
        '00070e151c232a31383f464d545b626970777e858c939aa1a8afb6bdc4cbd2d9e0e7eef5fc030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dce3eaf1f8ff060d141b222930373e454c535a61686f767d848b9299a0a7aeb5bcc3cad1d8dfe6edf4fb020910171e252c333a41484f565d646b727980878e959ca3aab1b8bfc6cdd4dbe2e9f0f7fe050c131a21282f363d444b525960676e757c838a91989fa6adb4bbc2c9d0d7dee5ecf3fa01080f161d242b323940474e555c636a71787f868d949ba2a9b0b7bec5ccd3dae1e8eff6fd040b121920272e353c434a51585f666d747b828990979ea5acb3bac1c8cfd6dde4ebf2f900070e151c232a31383f464d545b626970777e858c939aa1a8afb6bdc4cbd2d9e0e7eef5fc030a11181f262d'
      )
    ],
    '12c07f00609601000e1c2a38465462707e8c9aa8b6c4d2e0eefc0a19273543515f6d7b8997a5b3c1cfddebf9071422303e4c5a68768492a0aebccad8e6f402111f2d3b49576573818f9dabb9c7d5e3f1ff0d1a28364452606e7c8a98a6b4c2d0deecfa08172533414f5d6b798795a3b1bfcddbe9f70512202e3c4a58667482909eacbac8d6e4f2000f1d2b39475563717f8d9ba9b7c5d3e1effd0b18263442505e6c7a8896a4b2c0cedceaf8061523313f4d5b69778593a1afbdcbd9e7f503101e2c3a48566472808e9caab8c6d4e2f0fe0c1b29374553616f7d8b99a7b5c3d1dfedfb09162432404e5c6a788694a2b0beccdae8f60413212f3d4b59677583919fadbbc9d7e5f3010e1c2a38465462707e8c9aa8b6c4d2e0eefc0a19273543515f6d7b8997a5b3c1cfddebf9071422303e4c5a00',
    2457,
  ),
  RustVector(
    'optional',
    [
      ('opt<u5>', null),
      ('opt<u5>', '17'),
      ('opt<astr3>', 'ab'),
      ('opt<f64>', null),
      ('opt<bool>', 'true')
    ],
    'c6254ccc',
    32,
  ),
  RustVector(
    'fixed_array',
    [
      ('fa3<u5>', ['1', '2', '31']),
      ('fa2<di32d3>', ['-10', '0']),
      ('fa4<bool>', ['true', 'false', 'false', 'true']),
      (
        'fa2<fa2<i3>>',
        [
          ['-3', '2'],
          ['0', '-1']
        ]
      )
    ],
    '41fc35c82318',
    47,
  ),
  RustVector(
    'dyn_array',
    [
      ('da3<u5>', <Object?>[]),
      ('da3<u5>', ['1', '2', '3']),
      ('da4<f32>', ['3f800000', 'bf000000']),
      ('da2<opt<i7>>', [null, '-100']),
      (
        'da3<da2<bool>>',
        [
          ['true'],
          <Object?>[],
          ['false', 'true', 'true', 'false', 'true']
        ]
      ),
      (
        'da1<astr4>',
        ['', 'x', 'xx', 'xxx', 'xxxx', 'xxxxx', 'xxxxxx', 'xxxxxxx', 'xxxxxxxx']
      ),
      (
        'da8<u64>',
        ['18446744073709551615', '18446744073709551615', '18446744073709551615']
      )
    ],
    '4e10a3000000fc010000f82d93cfb4b1d70c78051ede81878797e0e1e1e12d78787878780d1e1e1e1e1ede8387878787878717e1e1e1e1e1e1e1e11df0ffffffffffffffffffffffffffffffffffffffffffffff0f',
    676,
  ),
  RustVector(
    'mixed1',
    [
      ('bool', 'true'),
      ('u3', '5'),
      ('i12', '-2047'),
      ('du32d3', '10'),
      ('f32', '3fa00000'),
      ('astr4', 'Hi!'),
      ('fp10[-5,100]', '4040a66666666666>4040a0d8a279db65'),
      ('f64', 'c01e000000000000'),
      ('bin3', 'deadbeef'),
      ('di64d8', '-1234567890123'),
      ('u64', '18364758544493064720'),
      ('bool', 'false')
    ],
    'fbff6a000080fe1c48692175010000000000780027ef56dff79713dc1ff7630008192a3b4c5d6e7f',
    320,
  ),
  RustVector(
    'mixed2',
    [
      ('u7', '99'),
      ('opt<f64>', '0010000000000000'),
      ('da2<du16d4>', ['0', '65535', '16']),
      ('i40', '-549755813887'),
      ('du50d9', '562949953421312'),
      ('bool', 'true'),
      ('fp13[-100,100]', 'bf847ae147ae147b>bf89025838454000')
    ],
    'e30000000000001000e7ffffc3f0ffffffff2f800002082080f0ff03',
    219,
  ),
  RustVector(
    'mixed3_single_bit',
    [('bool', 'true')],
    '01',
    1,
  ),
];
