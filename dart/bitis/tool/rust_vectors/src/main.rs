// Generates Dart test fixtures (test/rust_vectors.dart) from the Rust bitis
// runtime. Only VALID values are serialized.
use bitis_lib::*;

trait DS: BiserdiTrait {
    fn spec() -> String;
    fn val(&self) -> String;
}

fn q(s: &str) -> String { format!("'{}'", s) }

impl DS for bool {
    fn spec() -> String { "bool".into() }
    fn val(&self) -> String { q(&self.to_string()) }
}

macro_rules! ds_fixed {
    ($t:ty, $p:expr) => {
        impl<const X: u64> DS for IntWithGivenBitSize<$t, X> {
            fn spec() -> String { format!("{}{}", $p, X) }
            fn val(&self) -> String { q(&self.val.to_string()) }
        }
    };
}
ds_fixed!(u16, "u"); ds_fixed!(u32, "u"); ds_fixed!(u64, "u");
ds_fixed!(i16, "i"); ds_fixed!(i32, "i"); ds_fixed!(i64, "i");

macro_rules! ds_dyn {
    ($t:ty, $p:expr) => {
        impl<const X: u8, const Y: u8> DS for DynInteger<$t, X, Y> {
            fn spec() -> String { format!("{}{}d{}", $p, X, Y) }
            fn val(&self) -> String { q(&self.val.to_string()) }
        }
    };
}
ds_dyn!(u16, "du"); ds_dyn!(u32, "du"); ds_dyn!(u64, "du");
ds_dyn!(i16, "di"); ds_dyn!(i32, "di"); ds_dyn!(i64, "di");

impl DS for f32 {
    fn spec() -> String { "f32".into() }
    fn val(&self) -> String { q(&format!("{:08x}", self.to_bits())) }
}
impl DS for f64 {
    fn spec() -> String { "f64".into() }
    fn val(&self) -> String { q(&format!("{:016x}", self.to_bits())) }
}

fn fpv(v: &FixPrecisionVal) -> String {
    match v {
        FixPrecisionVal::Underflow => "U".into(),
        FixPrecisionVal::Overflow => "O".into(),
        FixPrecisionVal::Value(x) => format!("{:016x}", x.to_bits()),
    }
}
impl<const N: u8, const MIN: i64, const MAX: i64> DS for FixPrecisionMinMax<N, MIN, MAX> {
    fn spec() -> String { format!("fp{}[{},{}]", N, MIN, MAX) }
    fn val(&self) -> String {
        // input > decoded value (round trip through the Rust runtime)
        let mut b = Biseri::new();
        self.bit_serialize(&mut b).unwrap();
        b.finish_add_data().unwrap();
        let mut d = Bides::from_biseri(&b);
        let (o, _) = Self::bit_deserialize(1, &mut d).unwrap();
        q(&format!("{}>{}", fpv(&self.val), fpv(&o.val)))
    }
}

fn dart_str(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || " !#%&()*+,-./:;<=>?@[]^_`{|}~\"".contains(c) {
            o.push(c);
        } else {
            o.push_str(&format!("\\x{:02x}", c as u32));
        }
    }
    q(&o)
}
impl<const Y: u8> DS for BitisAString<Y> {
    fn spec() -> String { format!("astr{}", Y) }
    fn val(&self) -> String { dart_str(&self.val_into()) }
}
fn hex(v: &[u8]) -> String { v.iter().map(|b| format!("{:02x}", b)).collect() }
impl<const Y: u8> DS for Binary<Y> {
    fn spec() -> String { format!("bin{}", Y) }
    fn val(&self) -> String { q(&hex(&self.val)) }
}
impl<T: DS + Default> DS for BitisOption<T> {
    fn spec() -> String { format!("opt<{}>", T::spec()) }
    fn val(&self) -> String {
        match &self.val { None => "null".into(), Some(v) => v.val() }
    }
}
impl<T: DS + Default + Clone + std::fmt::Debug, const N: usize> DS for FixedArray<T, N> {
    fn spec() -> String { format!("fa{}<{}>", N, T::spec()) }
    fn val(&self) -> String {
        if self.val.is_empty() { return "<Object?>[]".into(); }
        format!("[{}]", self.val.iter().map(|v| v.val()).collect::<Vec<_>>().join(", "))
    }
}
impl<T: DS + Default + Clone, const Y: u8> DS for DynArray<T, Y> {
    fn spec() -> String { format!("da{}<{}>", Y, T::spec()) }
    fn val(&self) -> String {
        if self.val.is_empty() { return "<Object?>[]".into(); }
        format!("[{}]", self.val.iter().map(|v| v.val()).collect::<Vec<_>>().join(", "))
    }
}

struct Seq { name: String, ser: Biseri, items: Vec<String> }
impl Seq {
    fn new(name: &str) -> Self { Seq { name: name.into(), ser: Biseri::new(), items: vec![] } }
    fn add<T: DS>(mut self, v: T) -> Self {
        v.bit_serialize(&mut self.ser).unwrap();
        self.items.push(format!("('{}', {})", T::spec(), v.val()));
        self
    }
    fn done(mut self, out: &mut Vec<String>) {
        let s = self.ser.finish_add_data().unwrap();
        out.push(format!(
            "  RustVector(\n    '{}',\n    [{}],\n    '{}',\n    {},\n  ),",
            self.name, self.items.join(", "), hex(self.ser.get_data_ref()), s.total_bits));
    }
}

type U<const X: u64> = IntWithGivenBitSize<u16, X>;
type U32<const X: u64> = IntWithGivenBitSize<u32, X>;
type U64<const X: u64> = IntWithGivenBitSize<u64, X>;
type I<const X: u64> = IntWithGivenBitSize<i16, X>;
type I32<const X: u64> = IntWithGivenBitSize<i32, X>;
type I64<const X: u64> = IntWithGivenBitSize<i64, X>;
type FP<const N: u8, const A: i64, const B: i64> = FixPrecisionMinMax<N, A, B>;

fn fp<const N: u8, const A: i64, const B: i64>(x: f64) -> FP<N, A, B> {
    FP::<N, A, B> { val: FixPrecisionVal::Value(x) }
}
fn astr<const Y: u8>(s: &str) -> BitisAString<Y> { BitisAString::val_from(&s.to_string()) }

fn main() {
    let mut o: Vec<String> = vec![];

    // --- bool / fixed unsigned
    Seq::new("bool").add(true).add(false).add(true).done(&mut o);
    Seq::new("uint_1").add(U::<1>::new(0)).add(U::<1>::new(1)).done(&mut o);
    Seq::new("uint_5").add(U::<5>::new(0)).add(U::<5>::new(1)).add(U::<5>::new(17)).add(U::<5>::new(31)).done(&mut o);
    Seq::new("uint_8").add(U::<8>::new(255)).add(U::<8>::new(128)).done(&mut o);
    Seq::new("uint_16").add(U::<16>::new(65535)).add(U::<16>::new(256)).done(&mut o);
    Seq::new("uint_17").add(U32::<17>::new((1 << 17) - 1)).add(U32::<17>::new(1 << 16)).done(&mut o);
    Seq::new("uint_31").add(U32::<31>::new((1 << 31) - 1)).done(&mut o);
    Seq::new("uint_32").add(U32::<32>::new(u32::MAX)).add(U32::<32>::new(1 << 31)).add(U32::<32>::new(0x12345678)).done(&mut o);
    Seq::new("uint_33").add(U64::<33>::new((1 << 33) - 1)).add(U64::<33>::new(1 << 32)).done(&mut o);
    Seq::new("uint_53").add(U64::<53>::new((1 << 53) - 1)).add(U64::<53>::new(0x1234_5678_9abc_d)).done(&mut o);
    Seq::new("uint_63").add(U64::<63>::new((1 << 63) - 1)).add(U64::<63>::new(1 << 62)).done(&mut o);
    Seq::new("uint_64").add(U64::<64>::new(u64::MAX)).add(U64::<64>::new(1 << 63)).add(U64::<64>::new(0)).add(U64::<64>::new(0x0123_4567_89ab_cdef)).done(&mut o);

    // --- fixed signed (sign-magnitude)
    Seq::new("int_1").add(I::<1>::new(-1)).add(I::<1>::new(0)).add(I::<1>::new(1)).done(&mut o);
    Seq::new("int_5").add(I::<5>::new(-31)).add(I::<5>::new(31)).add(I::<5>::new(-1)).add(I::<5>::new(0)).done(&mut o);
    Seq::new("int_15").add(I::<15>::new(-32767)).add(I::<15>::new(32767)).done(&mut o);
    Seq::new("int_16").add(I::<16>::new(-32767)).add(I::<16>::new(32767)).add(I::<16>::new(-256)).done(&mut o);
    Seq::new("int_17").add(I32::<17>::new(-131071)).add(I32::<17>::new(131071)).done(&mut o);
    Seq::new("int_32").add(I32::<32>::new(-i32::MAX)).add(I32::<32>::new(i32::MAX)).add(I32::<32>::new(-5)).done(&mut o);
    Seq::new("int_33").add(I64::<33>::new(-((1 << 33) - 1))).add(I64::<33>::new((1 << 33) - 1)).done(&mut o);
    Seq::new("int_53").add(I64::<53>::new(-((1 << 53) - 1))).add(I64::<53>::new(1 << 52)).done(&mut o);
    Seq::new("int_63").add(I64::<63>::new(-i64::MAX)).add(I64::<63>::new(i64::MAX)).done(&mut o);
    Seq::new("int_64").add(I64::<64>::new(-i64::MAX)).add(I64::<64>::new(i64::MAX)).add(I64::<64>::new(-1)).done(&mut o);

    // --- dyn unsigned
    Seq::new("uint_32d3").add(DynInteger::<u32, 32, 3>::new(0)).add(DynInteger::<u32, 32, 3>::new(10))
        .add(DynInteger::<u32, 32, 3>::new(7)).add(DynInteger::<u32, 32, 3>::new(8))
        .add(DynInteger::<u32, 32, 3>::new(u32::MAX)).done(&mut o);
    Seq::new("uint_8d8").add(DynInteger::<u16, 8, 8>::new(0)).add(DynInteger::<u16, 8, 8>::new(255)).done(&mut o);
    Seq::new("uint_16d5").add(DynInteger::<u16, 16, 5>::new(65535)).add(DynInteger::<u16, 16, 5>::new(32768)).add(DynInteger::<u16, 16, 5>::new(31)).add(DynInteger::<u16, 16, 5>::new(32)).done(&mut o);
    Seq::new("uint_1d1").add(DynInteger::<u16, 1, 1>::new(0)).add(DynInteger::<u16, 1, 1>::new(1)).done(&mut o);
    Seq::new("uint_32d32").add(DynInteger::<u32, 32, 32>::new(u32::MAX)).add(DynInteger::<u32, 32, 32>::new(1)).done(&mut o);
    Seq::new("uint_40d6").add(DynInteger::<u64, 40, 6>::new((1 << 40) - 1)).add(DynInteger::<u64, 40, 6>::new(1 << 36)).done(&mut o);
    Seq::new("uint_63d7").add(DynInteger::<u64, 63, 7>::new((1 << 63) - 1)).add(DynInteger::<u64, 63, 7>::new(128)).done(&mut o);
    Seq::new("uint_64d8").add(DynInteger::<u64, 64, 8>::new(u64::MAX)).add(DynInteger::<u64, 64, 8>::new(1 << 63)).add(DynInteger::<u64, 64, 8>::new(0)).done(&mut o);
    Seq::new("uint_64d33").add(DynInteger::<u64, 64, 33>::new(u64::MAX)).add(DynInteger::<u64, 64, 33>::new(1 << 33)).done(&mut o);

    // --- dyn signed
    Seq::new("int_32d3").add(DynInteger::<i32, 32, 3>::new(0)).add(DynInteger::<i32, 32, 3>::new(-10))
        .add(DynInteger::<i32, 32, 3>::new(10)).add(DynInteger::<i32, 32, 3>::new(i32::MAX))
        .add(DynInteger::<i32, 32, 3>::new(-i32::MAX)).done(&mut o);
    Seq::new("int_16d4").add(DynInteger::<i16, 16, 4>::new(32767)).add(DynInteger::<i16, 16, 4>::new(-32767)).done(&mut o);
    Seq::new("int_8d3").add(DynInteger::<i16, 8, 3>::new(255)).add(DynInteger::<i16, 8, 3>::new(-255)).add(DynInteger::<i16, 8, 3>::new(-1)).done(&mut o);
    Seq::new("int_64d8").add(DynInteger::<i64, 64, 8>::new(i64::MAX)).add(DynInteger::<i64, 64, 8>::new(-i64::MAX)).add(DynInteger::<i64, 64, 8>::new(0)).done(&mut o);
    Seq::new("int_48d10").add(DynInteger::<i64, 48, 10>::new((1 << 48) - 1)).add(DynInteger::<i64, 48, 10>::new(-(1 << 40))).done(&mut o);

    // --- floats
    Seq::new("float")
        .add(0.0f32).add(-0.0f32).add(1.5f32).add(-2.25f32).add(f32::NAN)
        .add(f32::INFINITY).add(f32::NEG_INFINITY).add(f32::MIN_POSITIVE)
        .add(f32::from_bits(1)).add(f32::MAX).add(0.1f32).done(&mut o);
    Seq::new("double")
        .add(0.0f64).add(-0.0f64).add(std::f64::consts::PI).add(f64::NAN)
        .add(f64::INFINITY).add(f64::NEG_INFINITY).add(f64::from_bits(1))
        .add(f64::MAX).add(-1e300).add(0.1f64).done(&mut o);

    // --- fixed point
    Seq::new("fp10[-5,100]").add(fp::<10, -5, 100>(-5.0)).add(fp::<10, -5, 100>(100.0))
        .add(fp::<10, -5, 100>(0.0)).add(fp::<10, -5, 100>(47.5)).add(fp::<10, -5, 100>(12.345))
        .add(FP::<10, -5, 100> { val: FixPrecisionVal::Underflow })
        .add(FP::<10, -5, 100> { val: FixPrecisionVal::Overflow }).done(&mut o);
    // S = 253 = R: t = v + 1, exact .5 ties
    Seq::new("fp8[0,253] ties").add(fp::<8, 0, 253>(0.5)).add(fp::<8, 0, 253>(1.5))
        .add(fp::<8, 0, 253>(2.5)).add(fp::<8, 0, 253>(252.5)).add(fp::<8, 0, 253>(0.49999999999999994))
        .add(fp::<8, 0, 253>(-0.0)).add(fp::<8, 0, 253>(253.0)).done(&mut o);
    Seq::new("fp2[0,1]").add(fp::<2, 0, 1>(0.0)).add(fp::<2, 0, 1>(1.0)).add(fp::<2, 0, 1>(0.5))
        .add(fp::<2, 0, 1>(0.4999)).done(&mut o);
    Seq::new("fp16[-40,125]").add(fp::<16, -40, 125>(21.37)).add(fp::<16, -40, 125>(-39.999))
        .add(fp::<16, -40, 125>(124.9999)).done(&mut o);
    Seq::new("fp53[-1000000,1000000]").add(fp::<53, -1000000, 1000000>(0.1))
        .add(fp::<53, -1000000, 1000000>(-999999.123456789)).add(fp::<53, -1000000, 1000000>(1000000.0))
        .add(FP::<53, -1000000, 1000000> { val: FixPrecisionVal::Overflow }).done(&mut o);
    Seq::new("fp33[-5000000000,5000000000]").add(fp::<33, -5000000000, 5000000000>(123456789.5))
        .add(fp::<33, -5000000000, 5000000000>(-5000000000.0)).done(&mut o);

    // --- strings / binary
    Seq::new("astr").add(astr::<4>("")).add(astr::<4>("hello")).add(astr::<3>("\x00\x7f!'$\\"))
        .add(astr::<8>(&"0123456789abcdefghijklmnopqrstuvwxyz".repeat(9))).done(&mut o);
    Seq::new("binary").add(Binary::<3>::new(vec![])).add(Binary::<3>::new(vec![0, 255, 1, 128]))
        .add(Binary::<8>::new((0..300).map(|i| (i * 7 % 256) as u8).collect())).done(&mut o);

    // --- optional / arrays
    Seq::new("optional").add(BitisOption::<U<5>>::new_none()).add(BitisOption::new_some(U::<5>::new(17)))
        .add(BitisOption::new_some(astr::<3>("ab"))).add(BitisOption::<f64>::new_none())
        .add(BitisOption::new_some(true)).done(&mut o);
    Seq::new("fixed_array").add(FixedArray::<U<5>, 3> { val: [U::<5>::new(1), U::<5>::new(2), U::<5>::new(31)] })
        .add(FixedArray::<DynInteger<i32, 32, 3>, 2> { val: [DynInteger::new(-10), DynInteger::new(0)] })
        .add(FixedArray::<bool, 4> { val: [true, false, false, true] })
        .add(FixedArray::<FixedArray<I<3>, 2>, 2> { val: [FixedArray { val: [I::<3>::new(-3), I::<3>::new(2)] }, FixedArray { val: [I::<3>::new(0), I::<3>::new(-1)] }] })
        .done(&mut o);
    Seq::new("dyn_array").add(DynArray::<U<5>, 3> { val: vec![] })
        .add(DynArray::<U<5>, 3> { val: vec![U::<5>::new(1), U::<5>::new(2), U::<5>::new(3)] })
        .add(DynArray::<f32, 4> { val: vec![1.0, -0.5] })
        .add(DynArray::<BitisOption<I<7>>, 2> { val: vec![BitisOption::new_none(), BitisOption::new_some(I::<7>::new(-100))] })
        .add(DynArray::<DynArray<bool, 2>, 3> { val: vec![DynArray { val: vec![true] }, DynArray { val: vec![] }, DynArray { val: vec![false, true, true, false, true] }] })
        .add(DynArray::<BitisAString<4>, 1> { val: (0..9).map(|i| astr::<4>(&"x".repeat(i))).collect() })
        .add(DynArray::<U64<64>, 8> { val: vec![U64::<64>::new(u64::MAX); 3] })
        .done(&mut o);

    // --- unaligned mixed sequences
    Seq::new("mixed1").add(true).add(U::<3>::new(5)).add(I::<12>::new(-2047)).add(DynInteger::<u32, 32, 3>::new(10))
        .add(1.25f32).add(astr::<4>("Hi!")).add(fp::<10, -5, 100>(33.3)).add(-7.5f64)
        .add(Binary::<3>::new(vec![0xde, 0xad, 0xbe, 0xef])).add(DynInteger::<i64, 64, 8>::new(-1234567890123))
        .add(U64::<64>::new(0xfedc_ba98_7654_3210)).add(false).done(&mut o);
    Seq::new("mixed2").add(U::<7>::new(99)).add(BitisOption::new_some(f64::MIN_POSITIVE))
        .add(DynArray::<DynInteger<u16, 16, 4>, 2> { val: vec![DynInteger::new(0), DynInteger::new(65535), DynInteger::new(16)] })
        .add(I64::<40>::new(-549755813887)).add(DynInteger::<u64, 50, 9>::new(1 << 49)).add(true)
        .add(fp::<13, -100, 100>(-0.01)).done(&mut o);
    Seq::new("mixed3_single_bit").add(true).done(&mut o);

    println!("// GENERATED from the Rust reference runtime by dart/bitis/tool/rust_vectors");
    println!("// (see its Cargo.toml for how to regenerate). Do not edit. Only valid values.");
    println!("//");
    println!("// Item specs: bool, u<X>, i<X>, du<X>d<Y>, di<X>d<Y>, f32, f64 (values are");
    println!("// IEEE bit patterns in hex), fp<N>[min,max] (value 'input>decoded', U/O for");
    println!("// underflow/overflow, else f64 bits in hex), astr<Y>, bin<Y> (hex),");
    println!("// opt<T>, fa<N><T>, da<Y><T>.");
    println!();
    println!("// ignore_for_file: lines_longer_than_80_chars");
    println!();
    println!("/// One sequence of values serialized into one bit stream.");
    println!("class RustVector {{");
    println!("  /// Creates a vector.");
    println!("  const RustVector(this.name, this.items, this.hex, this.bits);");
    println!();
    println!("  /// Name.");
    println!("  final String name;");
    println!();
    println!("  /// (type spec, value) pairs in stream order.");
    println!("  final List<(String, Object?)> items;");
    println!();
    println!("  /// Encoded bytes as hex.");
    println!("  final String hex;");
    println!();
    println!("  /// Number of significant bits.");
    println!("  final int bits;");
    println!("}}");
    println!();
    println!("/// All vectors.");
    println!("const rustVectors = <RustVector>[");
    for l in o { println!("{}", l); }
    println!("];");
}
