// Regression tests for the bitis 0.11 C++ runtime hardening (decoding of untrusted
// input, error API, integer ranges, fixed point rounding, misc fixes).
#include <gtest/gtest.h>
#include <cmath>
#include <cstring>
#include <new>

#include "bitis_lib.h"
#include "bitis_lib.h"   // include guard
#include "test_helpers.h"

using namespace bitis_test;

namespace test_safety {

// ***************************************************************
// B1: bounds checks / sticky error
TEST(BITIS_Bounds, ReadPastEndSetsUnexpectedEnd) {
    BitisDeserializer des(nullptr, 0);
    EXPECT_EQ(des.read_bits(1), 0u);
    EXPECT_EQ(des.get_error(), BitisError::UnexpectedEnd);
    EXPECT_EQ(des.consumed_bits(), 0u);

    const uint8_t one[] = {0xA5};
    BitisDeserializer des2(one, 1);
    EXPECT_EQ(des2.read_bits(3), 0x5u);
    EXPECT_EQ(des2.remaining_bits(), 5u);
    EXPECT_EQ(des2.read_bits(6), 0u);      // only 5 bits left
    EXPECT_EQ(des2.get_error(), BitisError::UnexpectedEnd);
    EXPECT_EQ(des2.read_bits(1), 0u);      // sticky
    EXPECT_EQ(des2.consumed_bits(), 3u);
}

TEST(BITIS_Bounds, ReadBitsAcrossBytes) {
    const uint8_t data[] = {0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01, 0xFF};
    BitisDeserializer des(data, sizeof(data));
    EXPECT_EQ(des.read_bits(4), 0xFu);
    EXPECT_EQ(des.read_bits(64), 0xF0123456789ABCDEull);
    EXPECT_EQ(des.read_bits(4), 0xFu);
    EXPECT_FALSE(des.has_error());
    EXPECT_EQ(des.remaining_bits(), 0u);
}

TEST(BITIS_Bounds, BoolU8FromOneOfTwoBytes) {
    // analysis B1: {bool, u8} from 1 of 2 bytes must not decode
    Inner in; in.va = Inner::Va_T(-15); in.vb = true;
    auto bytes = serialize(in);
    ASSERT_EQ(bytes.size(), 1u);
    std::vector<uint8_t> empty;
    Inner out;
    EXPECT_EQ(try_deserialize(empty, out), BitisError::UnexpectedEnd);
}

TEST(BITIS_Bounds, OwningAndNonOwningDeserializer) {
    std::vector<uint8_t> *data = new std::vector<uint8_t>{0x12, 0x34};
    BitisDeserializer owning(*data);
    BitisDeserializer copy(owning);
    delete data;   // the owning deserializers keep their own copy
    EXPECT_EQ(copy.read_bits(16), 0x3412u);
    EXPECT_EQ(owning.read_bits(8), 0x12u);
    BitisDeserializer moved(std::move(owning));
    EXPECT_EQ(moved.read_bits(8), 0x34u);
    BitisDeserializer assigned(nullptr, 0);
    assigned = copy;
    EXPECT_EQ(assigned.consumed_bits(), 16u);

    const uint8_t raw[] = {0x01};
    BitisDeserializer non_owning(raw, 1);
    EXPECT_TRUE(non_owning.data_cache.empty());
    EXPECT_EQ(non_owning.read_bits(1), 1u);
}

// every truncated prefix of a valid encoding has to fail with UnexpectedEnd and
// must never read out of bounds (exact size heap copies for ASan)
template<typename T>
void check_all_truncations(const T &msg) {
    const auto full = serialize(msg);
    T decoded;
    BitisSize size{0, 0};
    ASSERT_EQ(try_deserialize(full, decoded, &size), BitisError::Ok);
    EXPECT_EQ(size.bytes, full.size());
    EXPECT_TRUE(decoded == msg);
    for (std::size_t len = 0; len < full.size(); len++) {
        auto *buf = new uint8_t[len ? len : 1];
        if (len) { std::memcpy(buf, full.data(), len); }
        T out;
        EXPECT_EQ(try_deserialize<T>(buf, len, out), BitisError::UnexpectedEnd) << "len " << len;
        EXPECT_EQ(try_deserialize_exact<T>(buf, len, out), BitisError::UnexpectedEnd) << "len " << len;
        std::vector<uint8_t> v(buf, buf + len);
        const auto legacy = deserialize_with_size<T>(v);
        EXPECT_EQ(legacy.bits, 0u);
        EXPECT_TRUE(legacy.data == T());
        delete[] buf;
    }
}

TEST(BITIS_Bounds, TruncatedInputAtEveryLength) {
    for (const auto &m : xl_vectors()) { check_all_truncations(m); }
    for (int i = 0; i < 5; i++) { check_all_truncations(all_types_sample(i)); }
    check_all_truncations(kv_sample());
    MsgKVOO kv; kv.key = MsgKVOO::Key_T("k");
    kv.value.set_oo<OO_MsgKvoo_Value::OO_IntVal>(OO_MsgKvoo_Value::OO_IntVal::OOType(-77777));
    check_all_truncations(kv);
}

// ***************************************************************
// error API
TEST(BITIS_ErrorApi, ErrorNames) {
    EXPECT_STREQ(bitis_error_name(BitisError::Ok), "Ok");
    EXPECT_STREQ(bitis_error_name(BitisError::UnexpectedEnd), "UnexpectedEnd");
    EXPECT_STREQ(bitis_error_name(BitisError::InvalidValue), "InvalidValue");
    EXPECT_STREQ(bitis_error_name(BitisError::InvalidDiscriminant), "InvalidDiscriminant");
    EXPECT_STREQ(bitis_error_name(BitisError::TrailingData), "TrailingData");
    EXPECT_STREQ(bitis_error_name(BitisError::ValueOutOfRange), "ValueOutOfRange");
}

TEST(BITIS_ErrorApi, TrySerializeOutOfRange) {
    // B13: uint_3 = 9 does not fit
    IntgralWithGivenBitSize<uint16_t, 3> v(9);
    std::vector<uint8_t> out{0xAA};
    EXPECT_EQ(try_serialize(v, out), BitisError::ValueOutOfRange);
    EXPECT_TRUE(out.empty());
    // legacy serialize keeps truncating silently
    EXPECT_EQ(serialize(v), std::vector<uint8_t>{0x01});

    BitisSize size{0, 0};
    v.value = 7;
    EXPECT_EQ(try_serialize(v, out, &size), BitisError::Ok);
    EXPECT_EQ(out, std::vector<uint8_t>{0x07});
    EXPECT_EQ(size.bits, 3u);
    EXPECT_EQ(size.bytes, 1u);

    XL m; m.u3 = XL::U3_T(8);
    EXPECT_EQ(try_serialize(m, out), BitisError::ValueOutOfRange);
}

TEST(BITIS_ErrorApi, TrailingDataAndSize) {
    Inner in; in.va = Inner::Va_T(5); in.vb = true;     // 7 bits
    auto bytes = serialize(in);
    ASSERT_EQ(bytes.size(), 1u);
    bytes.push_back(0xFF);

    Inner out; BitisSize size{0, 0};
    EXPECT_EQ(try_deserialize(bytes, out, &size), BitisError::Ok);
    EXPECT_TRUE(out == in);
    EXPECT_EQ(size.bits, 7u);
    EXPECT_EQ(size.bytes, 1u);

    Inner out2;
    EXPECT_EQ(try_deserialize_exact(bytes, out2), BitisError::TrailingData);
    EXPECT_TRUE(out2 == Inner());     // not modified on error
    bytes.pop_back();
    EXPECT_EQ(try_deserialize_exact(bytes, out2, &size), BitisError::Ok);
    EXPECT_TRUE(out2 == in);
    EXPECT_EQ(try_deserialize_exact(bytes.data(), bytes.size(), out2), BitisError::Ok);

    // padding bits are not checked
    bytes[0] |= 0x80;
    EXPECT_EQ(try_deserialize_exact(bytes, out2), BitisError::Ok);
}

TEST(BITIS_ErrorApi, OutputUntouchedOnError) {
    auto m = all_types_sample(1);
    auto bytes = serialize(m);
    bytes.resize(bytes.size() / 2);
    AllTypes out = all_types_sample(3);
    const AllTypes before = out;
    EXPECT_EQ(try_deserialize(bytes, out), BitisError::UnexpectedEnd);
    EXPECT_TRUE(out == before);
}

// ***************************************************************
// B2: count caps
TEST(BITIS_CountCaps, HugeDynArrayCount) {
    std::vector<uint8_t> d(8, 0xFF);
    DynArray<IntgralWithGivenBitSize<uint16_t, 6>, 3> out;
    EXPECT_EQ(try_deserialize(d, out), BitisError::UnexpectedEnd);
    EXPECT_TRUE((deserialize<DynArray<IntgralWithGivenBitSize<uint16_t, 6>, 3>>(d).values.empty()));

    // count 0x0FFFFF (~1M elements) with 4 bytes of input
    auto len = Bits().add(DynInteger<uint32_t, 32, 8>(0x0FFFFF)).bytes();
    EXPECT_EQ(try_deserialize(len, out), BitisError::UnexpectedEnd);

    // count = UINT32_MAX
    auto len2 = Bits().add(DynInteger<uint32_t, 32, 8>(UINT32_MAX)).add(0, 64).bytes();
    DynArray<Point, 8> pts;
    EXPECT_EQ(try_deserialize(len2, pts), BitisError::UnexpectedEnd);
}

TEST(BITIS_CountCaps, ExactFitIsAccepted) {
    // 3 elements of 6 bits: count (uint_32d3: 1+3+1 = 5 bits) + 18 bits = 23 bits
    auto bytes = Bits().add(DynInteger<uint32_t, 32, 3>(3)).add(0x3F, 6).add(0x01, 6).add(0x20, 6).bytes();
    ASSERT_EQ(bytes.size(), 3u);
    DynArray<IntgralWithGivenBitSize<uint16_t, 6>, 3> out;
    EXPECT_EQ(try_deserialize(bytes, out), BitisError::Ok);
    ASSERT_EQ(out.values.size(), 3u);
    EXPECT_EQ(out.values[2].value, 0x20);
    // 4 elements need 29 bits -> 4 * 6 > remaining
    auto bytes4 = Bits().add(DynInteger<uint32_t, 32, 3>(4)).add(0x3F, 6).add(0x01, 6).add(0x20, 6).bytes();
    EXPECT_EQ(try_deserialize(bytes4, out), BitisError::UnexpectedEnd);
}

TEST(BITIS_CountCaps, StringAndBinary) {
    auto bytes = Bits().add(DynInteger<uint32_t, 32, 4>(1000)).add(0x41, 8).add(0x42, 8).bytes();
    BitisAString<4> s;
    EXPECT_EQ(try_deserialize(bytes, s), BitisError::UnexpectedEnd);
    Binary<4> b;
    EXPECT_EQ(try_deserialize(bytes, b), BitisError::UnexpectedEnd);
    auto ok = Bits().add(DynInteger<uint32_t, 32, 4>(2)).add(0x41, 8).add(0x42, 8).bytes();
    EXPECT_EQ(try_deserialize(ok, s), BitisError::Ok);
    EXPECT_STREQ(s.get(), "AB");
    EXPECT_EQ(try_deserialize(ok, b), BitisError::Ok);
    EXPECT_EQ(b.get(), (std::vector<uint8_t>{0x41, 0x42}));
}

TEST(BITIS_MinBits, SpecTable) {
    EXPECT_EQ(BitisBool::min_bits(), 1u);
    EXPECT_EQ((IntgralWithGivenBitSize<uint16_t, 13>::min_bits()), 13u);
    EXPECT_EQ((IntgralWithGivenBitSize<int16_t, 5>::min_bits()), 6u);
    EXPECT_EQ((IntgralWithGivenBitSize<int64_t, 64>::min_bits()), 65u);
    EXPECT_EQ((DynInteger<uint32_t, 32, 4>::min_bits()), 1u);
    EXPECT_EQ((DynInteger<int16_t, 12, 4>::min_bits()), 2u);
    EXPECT_EQ(BitisFloatingPoint<float>::min_bits(), 32u);
    EXPECT_EQ(BitisFloatingPoint<double>::min_bits(), 64u);
    EXPECT_EQ((FixPrecisionMinMax<34, -90, 90>::min_bits()), 34u);
    EXPECT_EQ(BitisAString<4>::min_bits(), 1u);
    EXPECT_EQ(Binary<4>::min_bits(), 1u);
    EXPECT_EQ((DynArray<Point, 2>::min_bits()), 1u);
    EXPECT_EQ(BitisOptional<Point>::min_bits(), 1u);
    EXPECT_EQ((FixedArray<IntgralWithGivenBitSize<int16_t, 4>, 3>::min_bits()), 15u);
    EXPECT_EQ(Color::min_bits(), 1u);
    EXPECT_EQ(Empty::min_bits(), 0u);
    EXPECT_EQ(Point::min_bits(), 17u + 33u + 65u + 64u);
    EXPECT_EQ(Inner::min_bits(), 7u);
    EXPECT_EQ(OO_Xl_Val::min_bits(), 2u);              // 1 + min(1, 64, 7, 2)
    EXPECT_EQ(OO_AllTypes_Payload::min_bits(), 1u);    // 1 + Empty
    EXPECT_EQ(XL::min_bits(), 181u);
    EXPECT_EQ(AllTypes::min_bits(), 152u);
    EXPECT_EQ(MsgKVOO::min_bits(), 3u);
    EXPECT_EQ(bitis_min_bits<MsgKVMapOO>(), 1u);
    static_assert(bitis_min_bits<XL>() == 181, "constexpr");
    // a default XL is the minimal encoding except for the enum default (Green: 4 bits)
    EXPECT_EQ(serialize(XL()).size(), (181u + 3u + 7u) / 8u);
}

// ***************************************************************
// B4: DynInteger
TEST(BITIS_DynInteger, SpecExample) {
    // 10 as uint_32d3 -> 1 010 1 100 0 (9 bits)
    DynInteger<uint32_t, 32, 3> v(10);
    BitisSize size{0, 0};
    std::vector<uint8_t> out;
    ASSERT_EQ(try_serialize(v, out, &size), BitisError::Ok);
    EXPECT_EQ(size.bits, 9u);
    EXPECT_EQ(out, (std::vector<uint8_t>{0x35, 0x00}));
    // zero: 1 bit (unsigned), 2 bits (signed)
    EXPECT_EQ(try_serialize(DynInteger<uint32_t, 32, 3>(0), out, &size), BitisError::Ok);
    EXPECT_EQ(size.bits, 1u);
    EXPECT_EQ(try_serialize(DynInteger<int32_t, 32, 3>(0), out, &size), BitisError::Ok);
    EXPECT_EQ(size.bits, 2u);
}

TEST(BITIS_DynInteger, NoHangOnOutOfRange) {
    DynInteger<uint16_t, 4, 3> x(300);
    std::vector<uint8_t> out;
    EXPECT_EQ(try_serialize(x, out), BitisError::ValueOutOfRange);
    const auto legacy = serialize(x);    // truncates, terminates
    auto d = deserialize<DynInteger<uint16_t, 4, 3>>(legacy);
    EXPECT_EQ(d.value, 300 & 15);
}

TEST(BITIS_DynInteger, MaxBitsUsedUp) {
    // uint_4d3 = 15: flag, 3 bits, cont, 1 bit, no further continuation bit -> 6 bits
    BitisSize size{0, 0};
    std::vector<uint8_t> out;
    ASSERT_EQ(try_serialize(DynInteger<uint16_t, 4, 3>(15), out, &size), BitisError::Ok);
    EXPECT_EQ(size.bits, 6u);
    BitisDeserializer des(out.data(), out.size());
    auto r = DynInteger<uint16_t, 4, 3>::deserialize(des);
    EXPECT_EQ(r.data.value, 15);
    EXPECT_EQ(r.bits, 6u);
    EXPECT_EQ(des.consumed_bits(), 6u);
}

TEST(BITIS_DynInteger, NonMinimalEncodingAccepted) {
    // uint_8d4 = 3 as flag 1, 0011, cont 1, 0000
    auto bytes = Bits().add(1, 1).add(3, 4).add(1, 1).add(0, 4).bytes();
    DynInteger<uint16_t, 8, 4> out;
    BitisSize size{0, 0};
    EXPECT_EQ(try_deserialize(bytes, out, &size), BitisError::Ok);
    EXPECT_EQ(out.value, 3);
    EXPECT_EQ(size.bits, 10u);
}

TEST(BITIS_DynInteger, SignedRangeAndMinimum) {
    typedef DynInteger<int16_t, 16, 4> I16D4;
    EXPECT_EQ(I16D4::max_value(), 32767);
    EXPECT_EQ(I16D4::min_value(), -32767);
    std::vector<uint8_t> out;
    EXPECT_EQ(try_serialize(I16D4(INT16_MIN), out), BitisError::ValueOutOfRange);
    EXPECT_EQ(deserialize<I16D4>(serialize(I16D4(INT16_MIN))).value, 0);
    for (int16_t v : {int16_t(-32767), int16_t(-1), int16_t(0), int16_t(1), int16_t(32767)}) {
        ASSERT_EQ(try_serialize(I16D4(v), out), BitisError::Ok);
        I16D4 d;
        ASSERT_EQ(try_deserialize(out, d), BitisError::Ok);
        EXPECT_EQ(d.value, v);
    }
    // magnitude 32768 on the wire -> InvalidValue
    auto bad = Bits().add(0, 1).add(DynInteger<uint16_t, 16, 4>(32768)).bytes();
    I16D4 d;
    EXPECT_EQ(try_deserialize(bad, d), BitisError::InvalidValue);

    typedef DynInteger<int64_t, 64, 7> I64D7;
    EXPECT_EQ(try_serialize(I64D7(INT64_MIN), out), BitisError::ValueOutOfRange);
    EXPECT_EQ(deserialize<I64D7>(serialize(I64D7(INT64_MIN))).value, 0);
    ASSERT_EQ(try_serialize(I64D7(-INT64_MAX), out), BitisError::Ok);
    I64D7 d64;
    ASSERT_EQ(try_deserialize(out, d64), BitisError::Ok);
    EXPECT_EQ(d64.value, -INT64_MAX);

    typedef DynInteger<uint64_t, 64, 7> U64D7;
    ASSERT_EQ(try_serialize(U64D7(UINT64_MAX), out), BitisError::Ok);
    U64D7 du;
    ASSERT_EQ(try_deserialize(out, du), BitisError::Ok);
    EXPECT_EQ(du.value, UINT64_MAX);
}

// ***************************************************************
// B6 / B7: signed integers
TEST(BITIS_SignedInt, FullWidthTypesCompileAndRoundtrip) {
    Point p;
    p.px = Point::Px_T(-32767); p.py = Point::Py_T(-2147483647); p.pz = Point::Pz_T(-INT64_MAX);
    p.pw = Point::Pw_T(UINT64_MAX);
    std::vector<uint8_t> out;
    ASSERT_EQ(try_serialize(p, out), BitisError::Ok);
    Point d;
    ASSERT_EQ(try_deserialize(out, d), BitisError::Ok);
    EXPECT_TRUE(d == p);
    p.px.value = 32767; p.py.value = INT32_MAX; p.pz.value = INT64_MAX; p.pw.value = 0;
    ASSERT_EQ(try_serialize(p, out), BitisError::Ok);
    ASSERT_EQ(try_deserialize(out, d), BitisError::Ok);
    EXPECT_TRUE(d == p);

    EXPECT_EQ(Point::Px_T::max_value(), 32767);
    EXPECT_EQ(Point::Px_T::min_value(), -32767);
    EXPECT_EQ(Point::Pz_T::min_value(), -INT64_MAX);
    EXPECT_EQ(Point::Pw_T::min_value(), 0u);
    EXPECT_EQ(Point::Pw_T::max_value(), UINT64_MAX);
    EXPECT_EQ((IntgralWithGivenBitSize<int16_t, 5>::min_value()), -31);
}

TEST(BITIS_SignedInt, MinimumOfBaseTypeIsRejected) {
    std::vector<uint8_t> out;
    Point p; p.px.value = INT16_MIN;
    EXPECT_EQ(try_serialize(p, out), BitisError::ValueOutOfRange);
    p.px.value = 0; p.py.value = INT32_MIN;
    EXPECT_EQ(try_serialize(p, out), BitisError::ValueOutOfRange);
    p.py.value = 0; p.pz.value = INT64_MIN;
    EXPECT_EQ(try_serialize(p, out), BitisError::ValueOutOfRange);
    // legacy serialize: no UB, the truncated value still decodes (to 0)
    const auto legacy = serialize(p);
    Point d;
    EXPECT_EQ(try_deserialize(legacy, d), BitisError::Ok);
    EXPECT_EQ(d.pz.value, 0);
    // INT32_MIN in int_31 (old review case)
    EXPECT_EQ(try_serialize(IntgralWithGivenBitSize<int32_t, 31>(INT32_MIN), out), BitisError::ValueOutOfRange);
    EXPECT_EQ(serialize(IntgralWithGivenBitSize<int32_t, 31>(INT32_MIN)), (std::vector<uint8_t>{0x01, 0, 0, 0}));
}

TEST(BITIS_SignedInt, RangeOfSmallTypes) {
    typedef IntgralWithGivenBitSize<int16_t, 4> I4;
    std::vector<uint8_t> out;
    EXPECT_EQ(try_serialize(I4(-20), out), BitisError::ValueOutOfRange);
    EXPECT_EQ(try_serialize(I4(-15), out), BitisError::Ok);
    EXPECT_EQ(try_serialize(I4(16), out), BitisError::ValueOutOfRange);
    EXPECT_TRUE(I4::is_valid_value(15));
    EXPECT_FALSE(I4::is_valid_value(16));
    EXPECT_TRUE(I4(-15).is_valid());
}

TEST(BITIS_SignedInt, DecodeOutOfRangeAndNegativeZero) {
    // int_16: sign 0, magnitude 0x8000 -> InvalidValue
    auto bad = Bits().add(0, 1).add(0x8000, 16).bytes();
    IntgralWithGivenBitSize<int16_t, 16> v;
    EXPECT_EQ(try_deserialize(bad, v), BitisError::InvalidValue);
    auto bad64 = Bits().add(1, 1).add(0x8000000000000000ull, 64).bytes();
    IntgralWithGivenBitSize<int64_t, 64> v64;
    EXPECT_EQ(try_deserialize(bad64, v64), BitisError::InvalidValue);
    // negative zero decodes to 0
    auto nz = Bits().add(1, 1).add(0, 5).bytes();
    IntgralWithGivenBitSize<int16_t, 5> v5(7);
    EXPECT_EQ(try_deserialize(nz, v5), BitisError::Ok);
    EXPECT_EQ(v5.value, 0);
    auto nzd = Bits().add(1, 1).add(0, 1).bytes();
    DynInteger<int32_t, 32, 7> dv(7);
    EXPECT_EQ(try_deserialize(nzd, dv), BitisError::Ok);
    EXPECT_EQ(dv.value, 0);
    // zero is never encoded as negative
    EXPECT_EQ(serialize(IntgralWithGivenBitSize<int16_t, 5>(0)), std::vector<uint8_t>{0x00});
}

// ***************************************************************
// B5: fixed point
static double from_bits(uint64_t u) { double d; std::memcpy(&d, &u, sizeof(d)); return d; }

TEST(BITIS_FixedPoint, MatchesRustReference) {
    // values for which the old llround(t)+1 implementation differed from Rust
    EXPECT_EQ(hex(serialize(FixPrecisionMinMax<10, -2, 3>(from_bits(0xBFFB0644B386A4FCull)))), "4100");
    EXPECT_EQ(hex(serialize(FixPrecisionMinMax<10, -2, 3>(-1.689030362389814))), "4100");
    EXPECT_EQ(hex(serialize(FixPrecisionMinMax<7, 0, 100>(from_bits(0x4006666666666666ull)))), "05");
    EXPECT_EQ(hex(serialize(FixPrecisionMinMax<7, 0, 100>(from_bits(0x3FD9999999999998ull)))), "02");
    EXPECT_EQ(hex(serialize(FixPrecisionMinMax<12, -2000, 18000>(from_bits(0xC0877C374A5DF19Eull)))), "0101");
    EXPECT_EQ(hex(serialize(FixPrecisionMinMax<53, -1000000, 1000000>(0.0))), "00000000000010");
}

TEST(BITIS_FixedPoint, SpecialValues) {
    typedef FixPrecisionMinMax<10, -2, 3> FP;
    FP nan_v(NAN);
    EXPECT_TRUE(nan_v.is_underflow());
    EXPECT_EQ(nan_v.get_state(), Underflow);
    EXPECT_EQ(serialize(nan_v), (std::vector<uint8_t>{0x00, 0x00}));
    FP pinf(INFINITY);
    EXPECT_TRUE(pinf.is_overflow());
    EXPECT_EQ(serialize(pinf), (std::vector<uint8_t>{0xFF, 0x03}));
    FP ninf(-INFINITY);
    EXPECT_TRUE(ninf.is_underflow());
    EXPECT_EQ(serialize(FP(3.0000001)), (std::vector<uint8_t>{0xFF, 0x03}));
    EXPECT_EQ(serialize(FP(-2.0000001)), (std::vector<uint8_t>{0x00, 0x00}));
    // min -> raw 1, max -> raw 2^N - 2
    EXPECT_EQ(serialize(FP(-2.0)), (std::vector<uint8_t>{0x01, 0x00}));
    EXPECT_EQ(serialize(FP(3.0)), (std::vector<uint8_t>{0xFE, 0x03}));
    EXPECT_TRUE(FP(3.0).has_value());
    EXPECT_EQ(FP(3.0).get_value(), 3.0);
    EXPECT_EQ(FP(-2.0).get_value(), -2.0);
    // default is Overflow (unchanged)
    EXPECT_TRUE(FP().is_overflow());
    FP s(1.0); s.set_underflow(); EXPECT_TRUE(s.is_underflow());
    s.set_overflow(); EXPECT_TRUE(s.is_overflow());
}

TEST(BITIS_FixedPoint, DecodeFormula) {
    typedef FixPrecisionMinMax<10, -2, 3> FP;
    auto dec = [](uint64_t raw) {
        auto b = Bits().add(raw, 10).bytes();
        FP v; EXPECT_EQ(try_deserialize(b, v), BitisError::Ok); return v; };
    EXPECT_TRUE(dec(0).is_underflow());
    EXPECT_TRUE(dec(1023).is_overflow());
    EXPECT_EQ(dec(1).get_value(), -2.0);
    EXPECT_EQ(dec(1022).get_value(), 3.0);
    EXPECT_EQ(dec(512).get_value(), static_cast<double>(511) / 1021.0 * 5.0 + -2.0);
}

TEST(BITIS_FixedPoint, RoundHalfAwayFromZero) {
    // fp_2[0,1]: S = 1, R = 1 -> t = v + 1
    typedef FixPrecisionMinMax<2, 0, 1> FP2;
    EXPECT_EQ(serialize(FP2(0.5)), std::vector<uint8_t>{0x02});
    EXPECT_EQ(serialize(FP2(0.49)), std::vector<uint8_t>{0x01});
    EXPECT_EQ(FP2(0.5).get_value(), 1.0);
}

// ***************************************************************
// B12 / B25: discriminants, ASCII
TEST(BITIS_Discriminant, UnknownEnumIndex) {
    // Color has 6 values, Y = 2
    auto bytes = Bits().add(DynInteger<uint32_t, 32, 2>(6)).bytes();
    Color c;
    EXPECT_EQ(try_deserialize(bytes, c), BitisError::InvalidDiscriminant);
    EXPECT_TRUE(deserialize<Color>(bytes).is_enum<ColorEnum::Green>());   // legacy: default value
    auto ok = Bits().add(DynInteger<uint32_t, 32, 2>(5)).bytes();
    EXPECT_EQ(try_deserialize(ok, c), BitisError::Ok);
    EXPECT_TRUE(c.is_enum<ColorEnum::Yellow>());

    // invalid enum value set by hand -> ValueOutOfRange on encode
    c.value.value = 17;
    EXPECT_FALSE(c.is_valid());
    std::vector<uint8_t> out;
    EXPECT_EQ(try_serialize(c, out), BitisError::ValueOutOfRange);
    c.print(); printf("\n");    // unknown name must not crash
}

TEST(BITIS_Discriminant, UnknownOneofIndex) {
    // MsgKVOO: key astr_d4 "", oneof value(2) with 4 variants -> index 5 is unknown
    auto bytes = Bits().add(BitisAString<4>("")).add(DynInteger<uint32_t, 32, 2>(5)).add(0xFFFF, 16).bytes();
    MsgKVOO m;
    EXPECT_EQ(try_deserialize(bytes, m), BitisError::InvalidDiscriminant);
    // inside an array: no silent misalignment of the following fields
    auto arr = Bits().add(DynInteger<uint32_t, 32, 2>(2)).add(BitisAString<4>("a"))
        .add(DynInteger<uint32_t, 32, 2>(2)).add(1, 1)
        .add(BitisAString<4>("b")).add(DynInteger<uint32_t, 32, 2>(4)).add(0xFFFFFFFF, 32).bytes();
    MsgKVMapOO map;
    EXPECT_EQ(try_deserialize(arr, map), BitisError::InvalidDiscriminant);
}

TEST(BITIS_AString, NonAsciiByte) {
    auto bytes = Bits().add(DynInteger<uint32_t, 32, 4>(3)).add('a', 8).add(0xC3, 8).add('b', 8).bytes();
    BitisAString<4> s;
    EXPECT_EQ(try_deserialize(bytes, s), BitisError::InvalidValue);
    std::vector<uint8_t> out;
    EXPECT_EQ(try_serialize(BitisAString<4>("gr\xC3\xBC\xC3\x9F"), out), BitisError::ValueOutOfRange);
    // legacy: masked to 7 bit, stays decodable
    BitisAString<4> d;
    EXPECT_EQ(try_deserialize(serialize(BitisAString<4>("\xC1")), d), BitisError::Ok);
    EXPECT_STREQ(d.get(), "A");
}

TEST(BITIS_AString, GetIsNulTerminated) {
    auto bytes = serialize(BitisAString<4>("hello"));
    auto r = deserialize<BitisAString<4>>(bytes);
    EXPECT_EQ(strlen(r.get()), 5u);
    EXPECT_STREQ(r.get(), "hello");
    EXPECT_STREQ(r.c_str(), "hello");
    EXPECT_EQ(r.size(), 5u);
    EXPECT_EQ(r.length(), 5u);
    BitisAString<4> empty;
    EXPECT_STREQ(empty.get(), "");
    EXPECT_EQ(empty.size(), 0u);
    BitisAString<4> null_s(nullptr);
    EXPECT_STREQ(null_s.get(), "");
    BitisAString<4> from_vec(std::vector<uint8_t>{'x', 'y'});
    EXPECT_STREQ(from_vec.get(), "xy");
    EXPECT_TRUE(from_vec == BitisAString<4>("xy"));
    EXPECT_FALSE(from_vec == BitisAString<4>("x"));
    // the terminator is not serialized
    EXPECT_EQ(serialize(BitisAString<4>("A")), Bits().add(DynInteger<uint32_t, 32, 4>(1)).add('A', 8).bytes());
    // buffer modified through get_u8_vec(): get() restores the terminator
    from_vec.get_u8_vec()->pop_back();
    from_vec.get_u8_vec()->push_back('z');
    EXPECT_STREQ(from_vec.get(), "xyz");
    EXPECT_EQ(from_vec.size(), 3u);
}

// ***************************************************************
// B15 / B18 / B21: misc
TEST(BITIS_Misc, Binary) {
    Binary<4> b(std::vector<uint8_t>{1, 2, 3, 4});
    EXPECT_EQ(b.get_u8_vec()->size(), 4u);
    EXPECT_EQ(b.size(), 4u);
    auto bytes = serialize(b);
    EXPECT_EQ(hex(bytes), "4980C00001");
    auto r = deserialize<Binary<4>>(bytes);
    EXPECT_TRUE(r == b);
    EXPECT_FALSE(r != b);
    r.set({1, 2});
    EXPECT_FALSE(r == b);
    b.print(); printf("\n");
    const Binary<4> cb = b;
    EXPECT_EQ(cb.get_u8_vec()->size(), 4u);
}

TEST(BITIS_Misc, SerializerDefaultInitialized) {
    alignas(BitisSerializer) unsigned char buf[sizeof(BitisSerializer)];
    std::memset(buf, 0xA5, sizeof buf);
    auto *ser = new (buf) BitisSerializer;
    BitisBool(true).serialize(*ser);
    const auto size = ser->finalize();
    EXPECT_EQ(ser->data_cache, std::vector<uint8_t>{0x01});
    EXPECT_EQ(size.bits, 1u);
    EXPECT_EQ(ser->error, BitisError::Ok);
    ser->~BitisSerializer();
}

#if defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif
TEST(BITIS_Misc, AggregateInitWithEnumMember) {
    // BitisEnum() is no longer explicit -> members can be omitted in aggregate init
    XL x{XL::U3_T(3)};
    EXPECT_TRUE(x.col.is_enum<ColorEnum::Green>());
    AllTypes a{AllTypes::Flag_T(true)};
    EXPECT_TRUE(a.mode.is_enum<ModeEnum::On>());
    Color c = {};
    EXPECT_TRUE(c.is_enum<ColorEnum::Green>());
}
#if defined(__GNUC__)
#pragma GCC diagnostic pop
#endif

TEST(BITIS_Misc, FloatBitPattern) {
    EXPECT_EQ(hex(serialize(BitisFloatingPoint<float>(1.0f))), "0000803F");
    EXPECT_EQ(hex(serialize(BitisFloatingPoint<double>(-2.0))), "00000000000000C0");
    auto r = deserialize<BitisFloatingPoint<double>>(unhex("00000000000000C0"));
    EXPECT_EQ(r.value, -2.0);
}

TEST(BITIS_Misc, ConstSerializeAndPrint) {
    const auto m = all_types_sample(1);
    const auto bytes = serialize(m);
    std::vector<uint8_t> out;
    EXPECT_EQ(try_serialize(m, out), BitisError::Ok);
    EXPECT_EQ(out, bytes);
    m.print(-1); printf("\n");
    const BitisAString<4> s("const");
    EXPECT_STREQ(s.get(), "const");
}

TEST(BITIS_Misc, LegacyApiUnchanged) {
    auto m = all_types_sample(3);
    std::vector<uint8_t> bytes = serialize(m);
    auto d = deserialize<AllTypes>(bytes);
    EXPECT_TRUE(d == m);
    auto ds = deserialize_with_size<AllTypes>(bytes);
    EXPECT_TRUE(ds.data == m);
    BitisSerializer ser;
    const std::size_t bits = m.serialize(ser);
    const auto size = ser.finalize();
    EXPECT_EQ(ds.bits, bits);
    EXPECT_EQ(size.bits, bits);
    EXPECT_EQ(size.bytes, bytes.size());
    BitisDeserializer des(bytes);
    auto r = AllTypes::deserialize(des);
    EXPECT_EQ(r.bits, bits);
    EXPECT_FALSE(des.has_error());
}

}
