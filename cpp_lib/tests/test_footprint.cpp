// Tests of the footprint related API (bitis 0.11): static attribute name tables,
// try_serialize() into a caller-provided buffer, max_bits / max_bytes and the compact
// fixed point storage. The BITIS_NO_PRINT build is checked by no_print_check.cpp (a
// separate executable: print() differs between the two configurations).
#include <gtest/gtest.h>

#include <cstring>
#include <string>

#include "test_helpers.h"

namespace {
using namespace bitis_test;

// ***************************************************************
// attribute names

TEST(BITIS_Footprint, StaticAttributeNameTable) {
    static_assert(sizeof(AllTypes::msg_attr) / sizeof(AllTypes::msg_attr[0]) == 16, "one name per attribute");
    EXPECT_STREQ(AllTypes::msg_attr[0], "flag");
    EXPECT_STREQ(AllTypes::msg_attr[9], "ou");
    EXPECT_STREQ(AllTypes::msg_attr[15], "payload");
    EXPECT_STREQ(Inner::msg_attr[0], "va");
    EXPECT_STREQ(Inner::msg_attr[1], "vb");
    // the table is shared by all objects of the type
    const Inner a, b;
    EXPECT_EQ(&a.msg_attr[0], &b.msg_attr[0]);
}

TEST(BITIS_Footprint, MessagesHoldOnlyTheirAttributes) {
    // the generated struct has the layout of MsgT (the attributes), no name pointers
    EXPECT_EQ(sizeof(Inner), sizeof(Inner::MsgT));
    EXPECT_EQ(sizeof(Point), sizeof(Point::MsgT));
    EXPECT_EQ(sizeof(AllTypes), sizeof(AllTypes::MsgT));
    EXPECT_EQ(sizeof(XL), sizeof(XL::MsgT));
    // empty messages stay aggregates of size 1
    EXPECT_EQ(sizeof(Empty), 1u);
}

TEST(BITIS_Footprint, PrintOutputUnchanged) {
    Inner in;
    in.va = Inner::Va_T(-15);
    in.vb = BitisBool(true);
    testing::internal::CaptureStdout();
    in.print(-1);
    EXPECT_EQ(testing::internal::GetCapturedStdout(), "Inner{ va: -15 [int_5], vb: true, }");
    testing::internal::CaptureStdout();
    in.print();
    EXPECT_EQ(testing::internal::GetCapturedStdout(), "Inner{ \n  va: -15 [int_5],\n  vb: true,\n}");

    OO_Xl_Val o;
    o.set_oo<OO_Xl_Val::OO_Inner>(in);
    testing::internal::CaptureStdout();
    o.print(-1);
    EXPECT_EQ(testing::internal::GetCapturedStdout(),
        "Oneof = Inner (id:2) [enum_3] -> Inner{ va: -15 [int_5], vb: true, }");

    XL::Fp_T fp(0.5), under, over;
    under.set_underflow();
    testing::internal::CaptureStdout();
    fp.print(); printf("|"); under.print(); printf("|"); over.print();
    EXPECT_EQ(testing::internal::GetCapturedStdout(),
        "0.502449 [FP[-2, 3]]|Underflow [FP[-2, 3]]|Overflow [FP[-2, 3]]");
}

// ***************************************************************
// try_serialize into a caller-provided buffer

template<typename T>
void check_buffer_variant(const T &msg) {
    std::vector<uint8_t> expected;
    BitisSize vsize{0, 0};
    ASSERT_EQ(try_serialize(msg, expected, &vsize), BitisError::Ok);

    // exact fit
    std::vector<uint8_t> buf(expected.size() + 4, 0xEE);
    BitisSize size{0, 0};
    ASSERT_EQ(try_serialize(msg, buf.data(), expected.size(), &size), BitisError::Ok);
    EXPECT_EQ(size.bits, vsize.bits);
    EXPECT_EQ(size.bytes, expected.size());
    EXPECT_EQ(std::vector<uint8_t>(buf.begin(), buf.begin() + static_cast<long>(expected.size())), expected);
    // nothing written behind the message
    for (std::size_t i = expected.size(); i < buf.size(); i++) { EXPECT_EQ(buf[i], 0xEE); }

    // larger buffer
    std::fill(buf.begin(), buf.end(), 0);
    ASSERT_EQ(try_serialize(msg, buf.data(), buf.size(), &size), BitisError::Ok);
    EXPECT_EQ(size.bytes, expected.size());
    EXPECT_EQ(std::vector<uint8_t>(buf.begin(), buf.begin() + static_cast<long>(expected.size())), expected);

    // one byte too small: error, size reports what is needed, no write past cap
    if (!expected.empty()) {
        std::fill(buf.begin(), buf.end(), 0xEE);
        size = BitisSize{0, 0};
        EXPECT_EQ(try_serialize(msg, buf.data(), expected.size() - 1, &size), BitisError::BufferTooSmall);
        EXPECT_EQ(size.bytes, expected.size());
        EXPECT_EQ(size.bits, vsize.bits);
        EXPECT_EQ(buf[expected.size() - 1], 0xEE);
        EXPECT_EQ(try_serialize(msg, nullptr, 0), BitisError::BufferTooSmall);
    }
    // the bound from max_bytes always fits
    if (bitis_max_bytes<T>() != BITIS_UNBOUNDED) {
        EXPECT_LE(expected.size(), bitis_max_bytes<T>());
        EXPECT_LE(vsize.bits, bitis_max_bits<T>());
    }
}

TEST(BITIS_Footprint, BufferSerializeMatchesVector) {
    check_buffer_variant(Empty());
    check_buffer_variant(Inner());
    check_buffer_variant(Point());
    for (int i = 0; i < 4; i++) { check_buffer_variant(all_types_sample(i)); }
    for (const auto &x : xl_vectors()) { check_buffer_variant(x); }
    check_buffer_variant(kv_sample());
    check_buffer_variant(BitisAString<4>("abc"));
    check_buffer_variant(FixPrecisionMinMax<34, -90, 90>(51.5));
}

TEST(BITIS_Footprint, BufferSizedByMaxBytes) {
    Point p;
    p.px = Point::Px_T(-32767); p.py = Point::Py_T(INT32_MAX); p.pz = Point::Pz_T(INT64_MIN + 1); p.pw = Point::Pw_T(UINT64_MAX);
    uint8_t buf[bitis_max_bytes<Point>()];
    static_assert(sizeof(buf) == 23, "179 bits");
    BitisSize size{0, 0};
    ASSERT_EQ(try_serialize(p, buf, sizeof(buf), &size), BitisError::Ok);
    EXPECT_EQ(size.bits, 179u);
    EXPECT_EQ(std::vector<uint8_t>(buf, buf + sizeof(buf)), serialize(p));
    Point back;
    ASSERT_EQ(try_deserialize_exact(buf, sizeof(buf), back), BitisError::Ok);
    EXPECT_EQ(back, p);
    EXPECT_EQ(try_serialize(p, buf, sizeof(buf) - 1), BitisError::BufferTooSmall);
}

TEST(BITIS_Footprint, BufferErrorPrecedence) {
    // the first error wins; ValueOutOfRange in the first byte is reported
    AllTypes m = all_types_sample(0);
    m.u3 = AllTypes::U3_T(8);
    uint8_t buf[2];
    EXPECT_EQ(try_serialize(m, buf, sizeof(buf)), BitisError::ValueOutOfRange);
    EXPECT_STREQ(bitis_error_name(BitisError::BufferTooSmall), "BufferTooSmall");
}

// ***************************************************************
// max_bits (docs/WIRE_FORMAT.md section 4), values computed by hand

TEST(BITIS_Footprint, MaxBitsScalars) {
    EXPECT_EQ(bitis_max_bits<BitisBool>(), 1u);
    EXPECT_EQ((bitis_max_bits<IntgralWithGivenBitSize<uint8_t, 7>>()), 7u);
    EXPECT_EQ((bitis_max_bits<IntgralWithGivenBitSize<int64_t, 64>>()), 65u);
    EXPECT_EQ(bitis_max_bits<BitisFloatingPoint<float>>(), 32u);
    EXPECT_EQ(bitis_max_bits<BitisFloatingPoint<double>>(), 64u);
    EXPECT_EQ((bitis_max_bits<FixPrecisionMinMax<35, -180, 180>>()), 35u);
    // dynmax(X, Y) = X + ceil(X / Y); signed + 1
    EXPECT_EQ((bitis_max_bits<DynInteger<uint8_t, 8, 3>>()), 11u);
    EXPECT_EQ((bitis_max_bits<DynInteger<uint32_t, 32, 4>>()), 40u);
    EXPECT_EQ((bitis_max_bits<DynInteger<uint64_t, 64, 64>>()), 65u);
    EXPECT_EQ((bitis_max_bits<DynInteger<uint64_t, 64, 7>>()), 74u);
    EXPECT_EQ((bitis_max_bits<DynInteger<int16_t, 16, 5>>()), 21u);
    EXPECT_EQ((bitis_max_bits<DynInteger<int64_t, 64, 8>>()), 73u);
    // modifiers
    EXPECT_EQ((bitis_max_bits<BitisOptional<IntgralWithGivenBitSize<uint8_t, 7>>>()), 8u);
    EXPECT_EQ((bitis_max_bits<FixedArray<IntgralWithGivenBitSize<int8_t, 4>, 3>>()), 15u);
    EXPECT_EQ((bitis_max_bits<DynArray<BitisBool, 3>>()), BITIS_UNBOUNDED);
    EXPECT_EQ((bitis_max_bits<BitisOptional<DynArray<BitisBool, 3>>>()), BITIS_UNBOUNDED);
    EXPECT_EQ((bitis_max_bits<FixedArray<Binary<4>, 2>>()), BITIS_UNBOUNDED);
    EXPECT_EQ(bitis_max_bits<Binary<4>>(), BITIS_UNBOUNDED);
    EXPECT_EQ(bitis_max_bits<BitisAString<4>>(), BITIS_UNBOUNDED);
    // max_bytes = ceil(bits / 8)
    EXPECT_EQ(bitis_max_bytes<BitisBool>(), 1u);
    EXPECT_EQ((bitis_max_bytes<DynInteger<uint32_t, 32, 4>>()), 5u);
    EXPECT_EQ((bitis_max_bytes<DynInteger<uint64_t, 64, 7>>()), 10u);
    EXPECT_EQ(bitis_max_bytes<Binary<4>>(), BITIS_UNBOUNDED);
    // saturation
    EXPECT_EQ(bitis_helper::max_bits_add(BITIS_UNBOUNDED - 1, 2), BITIS_UNBOUNDED);
    EXPECT_EQ(bitis_helper::max_bits_mul(BITIS_UNBOUNDED / 2, 3), BITIS_UNBOUNDED);
    EXPECT_EQ(bitis_helper::max_bits_mul(BITIS_UNBOUNDED, 0), 0u);
}

TEST(BITIS_Footprint, MaxBitsGenerated) {
    // enum with k values: dyn(k - 1, 32, Y). Color: 6 values, Y=2 -> 5 = 01|01: 1+2+1+2+1
    EXPECT_EQ(bitis_max_bits<Color>(), 7u);
    // Mode: 3 values, Y=2 -> 2 = 10: 1+2+1
    EXPECT_EQ(bitis_max_bits<Mode>(), 4u);
    // messages: sum of the attributes
    EXPECT_EQ(bitis_max_bits<Empty>(), 0u);
    EXPECT_EQ(bitis_max_bytes<Empty>(), 0u);
    EXPECT_EQ(bitis_max_bits<Point>(), 17u + 33u + 65u + 64u);
    EXPECT_EQ(bitis_max_bits<Inner>(), 7u);
    EXPECT_EQ(Inner::max_bits(), 7u);
    // oneof val(3) {bool, double, Inner, int_32d7}: dyn(3, 32, 3) = 1+3+1, max(1, 64, 7, 1+32+5)
    EXPECT_EQ(bitis_max_bits<OO_Xl_Val>(), 5u + 64u);
    EXPECT_EQ(OO_Xl_Val::max_bits(), 69u);
    // unbounded parts
    EXPECT_EQ(bitis_max_bits<XL>(), BITIS_UNBOUNDED);
    EXPECT_EQ(bitis_max_bytes<AllTypes>(), BITIS_UNBOUNDED);
    // usable at compile time
    static_assert(bitis_max_bytes<Point>() == 23, "");
    static_assert(Inner::max_bits() == 7, "");
}

TEST(BITIS_Footprint, MaxBitsIsAnUpperBound) {
    // the largest values reach the bound for unsigned dyn integers
    std::vector<uint8_t> out;
    BitisSize size{0, 0};
    ASSERT_EQ(try_serialize(DynInteger<uint64_t, 64, 7>(UINT64_MAX), out, &size), BitisError::Ok);
    EXPECT_EQ(size.bits, 74u);
    ASSERT_EQ(try_serialize(Color::create_enum<ColorEnum::Yellow>(), out, &size), BitisError::Ok);
    EXPECT_EQ(size.bits, bitis_max_bits<Color>());
    OO_Xl_Val o;
    o.set_oo<OO_Xl_Val::OO_Big>(OO_Xl_Val::OO_Big::OOType(INT32_MIN + 1));
    ASSERT_EQ(try_serialize(o, out, &size), BitisError::Ok);
    EXPECT_LE(size.bits, bitis_max_bits<OO_Xl_Val>());
}

// ***************************************************************
// compact fixed point

typedef FixPrecisionMinMax<10, -2, 3> FP10;

TEST(BITIS_Footprint, FixedPointStorageSize) {
    EXPECT_EQ(sizeof(FixPrecisionMinMax<2, 0, 1>), 1u);
    EXPECT_EQ(sizeof(FixPrecisionMinMax<8, 0, 360>), 1u);
    EXPECT_EQ(sizeof(FixPrecisionMinMax<9, 0, 10000>), 2u);
    EXPECT_EQ(sizeof(FixPrecisionMinMax<16, 0, 1>), 2u);
    EXPECT_EQ(sizeof(FixPrecisionMinMax<17, 0, 1>), 4u);
    EXPECT_EQ(sizeof(FixPrecisionMinMax<32, -180, 180>), 4u);
    EXPECT_EQ(sizeof(FixPrecisionMinMax<34, -90, 90>), 8u);
    EXPECT_EQ(sizeof(FixPrecisionMinMax<64, -1000, 1000>), 8u);
    static_assert(std::is_same<FP10::RawT, uint16_t>::value, "");
}

TEST(BITIS_Footprint, FixedPointDefaultIsOverflow) {
    const FP10 d;
    EXPECT_TRUE(d.is_overflow());
    EXPECT_EQ(d.state(), Overflow);
    EXPECT_EQ(d.get_state(), Overflow);
    EXPECT_FALSE(d.has_value());
    EXPECT_EQ(d.raw(), 1023u);
    EXPECT_EQ(d.value(), 0.0);
    EXPECT_EQ(hex(serialize(d)), "FF03");
}

TEST(BITIS_Footprint, FixedPointStates) {
    FP10 v(-2.5);
    EXPECT_TRUE(v.is_underflow());
    EXPECT_EQ(v.raw(), 0u);
    EXPECT_EQ(v.get_value(), 0.0);
    v.set(3.5);
    EXPECT_TRUE(v.is_overflow());
    v.set(std::nan(""));
    EXPECT_TRUE(v.is_underflow());
    v.set(-2.0);
    EXPECT_EQ(v.state(), Ok);
    EXPECT_EQ(v.raw(), 1u);
    EXPECT_EQ(v.value(), -2.0);
    v.set(3.0);
    EXPECT_EQ(v.raw(), 1022u);
    EXPECT_EQ(v.value(), 3.0);
    v.set_underflow();
    EXPECT_EQ(v.state(), Underflow);
    v.set_overflow();
    EXPECT_EQ(v.state(), Overflow);
    v.set_raw(512);
    EXPECT_EQ(v.value(), static_cast<double>(511) / 1021.0 * 5.0 + -2.0);
    v.set_raw(0xFFFF);   // masked to N bits
    EXPECT_TRUE(v.is_overflow());
}

TEST(BITIS_Footprint, FixedPointRoundTripAndEquality) {
    for (int i = 0; i <= 1023; i++) {
        FP10 v;
        v.set_raw(static_cast<uint16_t>(i));
        const auto bytes = serialize(v);
        FP10 back;
        ASSERT_EQ(try_deserialize_exact(bytes, back), BitisError::Ok);
        EXPECT_EQ(back.raw(), v.raw());
        EXPECT_EQ(back, v);
        // re-quantizing the value gives the same raw value
        if (v.has_value()) { EXPECT_EQ(FP10(v.value()).raw(), v.raw()) << i; }
    }
    // equality on the raw value
    EXPECT_EQ(FP10(1.0), FP10(1.001));
    EXPECT_NE(FP10(1.0), FP10(1.01));
    FP10 u1(-100.0), u2;
    u2.set_underflow();
    EXPECT_EQ(u1, u2);
    EXPECT_NE(u1, FP10());
    EXPECT_EQ(FP10(), FP10(100.0));
    // the legacy static helpers keep their meaning
    double val = 0;
    FixPrecisionMinMaxEnum st = Ok;
    FP10::u64_to_val(FP10::val_to_u64(0.5, Ok), val, st);
    EXPECT_EQ(st, Ok);
    EXPECT_EQ(val, FP10(0.5).value());
    EXPECT_EQ(FP10::val_to_u64(0.0, Overflow), 1023u);
    EXPECT_EQ(FP10::val_to_u64(0.0, Underflow), 0u);
}

TEST(BITIS_Footprint, FixedPointWide) {
    typedef FixPrecisionMinMax<34, -90, 90> Lat;
    const Lat l(51.756349);
    EXPECT_NEAR(l.value(), 51.756349, 6e-9);
    Lat back;
    ASSERT_EQ(try_deserialize_exact(serialize(l), back), BitisError::Ok);
    EXPECT_EQ(back, l);
    EXPECT_EQ(Lat().raw(), (static_cast<uint64_t>(1) << 34) - 1);
    typedef FixPrecisionMinMax<64, -1000, 1000> W;
    EXPECT_TRUE(W().is_overflow());
    EXPECT_EQ(W().raw(), UINT64_MAX);
    EXPECT_EQ(W(0.5).state(), Ok);
}

}
