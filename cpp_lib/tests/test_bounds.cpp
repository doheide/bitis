// Tests of the bounded containers (spec 2.16): BoundedBinary, BoundedAString,
// BoundedDynArray and generated code using them (schema test_bounds.bitis).
#include <gtest/gtest.h>
#include <cstring>

#include "bitis_lib.h"
#include "test_helpers.h"
#include "gen/test_bounds.h"

using namespace bitis_test;

namespace test_bounds {

typedef BoundedBinary<3, 4> Bin4;
typedef BoundedAString<4, 8> Str8;
typedef BoundedAString<4, 5> Str5;
typedef BoundedDynArray<Str5, 2, 3> StrList3;

// ***************************************************************
// compile time properties
static_assert(sizeof(BoundedBinary<1, 255>::LenT) == 1, "uint8_t length for M <= 255");
static_assert(sizeof(BoundedBinary<1, 256>::LenT) == 2, "uint16_t length for M > 255");
static_assert(sizeof(BoundedAString<1, 65535>::LenT) == 2, "uint16_t length for M = 65535");
static_assert(Bin4::min_bits() == 1 && Str8::min_bits() == 1 && StrList3::min_bits() == 1, "min_bits");
static_assert(Bin4::capacity() == 4 && Str8::max_size() == 8 && StrList3::capacity() == 3, "capacity");
// max_bits (spec section 4): dyn(M, 32, Y) + M * max_bits(T)
static_assert(bitis_helper::dyn_bits(4, 32, 3) == 5, "dyn(4, 32, 3) = 1 + 3 + 1");
static_assert(Bin4::max_bits() == 5 + 4 * 8, "binary_d3[4]");
static_assert(Str8::max_bits() == 6 + 8 * 8, "astr_d4[8]: dyn(8, 32, 4) = 1 + 4 + 1");
static_assert(Str5::max_bits() == 6 + 5 * 8, "astr_d4[5]");
static_assert(StrList3::max_bits() == 4 + 3 * (6 + 5 * 8), "repeated_dyn_2[3] astr_d4[5]: dyn(3, 32, 2) = 1 + 2 + 1");
static_assert(BoundedAString<32, 65535>::max_bits() == 1 + 32 + 65535 * 8, "astr_d32[65535]");
static_assert(BoundedDynArray<BoundedBinary<1, 65535>, 1, 65535>::max_bits() ==
              bitis_helper::dyn_bits(65535, 32, 1) + 65535 * (bitis_helper::dyn_bits(65535, 32, 1) + 65535 * 8), "nested");
static_assert(bitis_min_bits<Bounded>() == 1 + 1 + 1 + 1 + 2 * 1 + 1 + 1 + (1 + 1) + 1, "min_bits of the message");

template<typename T>
static BitisError decode(const std::vector<uint8_t> &bytes, T &out) { return try_deserialize(bytes, out); }

// ***************************************************************
TEST(BITIS_BoundedBinary, ApiAndBound) {
    Bin4 b;
    EXPECT_TRUE(b.empty());
    EXPECT_TRUE(b.push_back(1));
    const uint8_t more[] = {2, 3, 4, 5};
    EXPECT_FALSE(b.append(more, 4));                 // would be 5 bytes
    EXPECT_EQ(b.size(), 1u);
    EXPECT_TRUE(b.append(more, 3));
    EXPECT_EQ(b.size(), 4u);
    EXPECT_FALSE(b.push_back(9));
    EXPECT_EQ(b[3], 4);
    EXPECT_FALSE(b.set(std::vector<uint8_t>{1, 2, 3, 4, 5}));
    EXPECT_EQ(b, Bin4({1, 2, 3, 4}));                // unchanged
    EXPECT_FALSE(b.resize(5));
    EXPECT_TRUE(b.resize(2));
    EXPECT_EQ(b, Bin4({1, 2}));
    EXPECT_TRUE(b.resize(3));
    EXPECT_EQ(b, Bin4({1, 2, 0}));
    std::size_t sum = 0;
    for (auto v : b) { sum += v; }
    EXPECT_EQ(sum, 3u);
    // constructors truncate
    EXPECT_EQ(Bin4(std::vector<uint8_t>{1, 2, 3, 4, 5, 6}).size(), 4u);
    EXPECT_EQ(Bin4(more, 4), Bin4({2, 3, 4, 5}));
    b.clear();
    EXPECT_EQ(b, Bin4());
}

TEST(BITIS_BoundedBinary, RoundTripAtMax) {
    const Bin4 b({0xDE, 0xAD, 0xBE, 0xEF});
    std::vector<uint8_t> bytes;
    BitisSize size{0, 0};
    ASSERT_EQ(try_serialize(b, bytes, &size), BitisError::Ok);
    EXPECT_EQ(size.bits, 5u + 32u);
    EXPECT_EQ(size.bits, Bin4::max_bits());
    // same encoding as the unbounded type
    EXPECT_EQ(bytes, serialize(Binary<3>(std::vector<uint8_t>{0xDE, 0xAD, 0xBE, 0xEF})));
    Bin4 out;
    ASSERT_EQ(try_deserialize_exact(bytes, out), BitisError::Ok);
    EXPECT_EQ(out, b);
}

TEST(BITIS_BoundedBinary, CountAboveBoundIsInvalidValue) {
    // count M + 1 with all data present
    Bits five; five.add(DynInteger<uint32_t, 32, 3>(5));
    for (int i = 0; i < 5; i++) { five.add(i, 8); }
    Bin4 out;
    EXPECT_EQ(decode(five.bytes(), out), BitisError::InvalidValue);
    // count M + 1 and the input too short: the bound is checked first
    Bits short_five; short_five.add(DynInteger<uint32_t, 32, 3>(5));
    EXPECT_EQ(decode(short_five.bytes(), out), BitisError::InvalidValue);
    Bits huge; huge.add(DynInteger<uint32_t, 32, 3>(UINT32_MAX));
    EXPECT_EQ(decode(huge.bytes(), out), BitisError::InvalidValue);
    // count M with too short input: count check of spec section 3
    Bits short_four; short_four.add(DynInteger<uint32_t, 32, 3>(4)).add(1, 8);
    EXPECT_EQ(decode(short_four.bytes(), out), BitisError::UnexpectedEnd);
    // legacy API: default value
    EXPECT_EQ(deserialize<Bin4>(five.bytes()), Bin4());
}

// ***************************************************************
TEST(BITIS_BoundedAString, ApiAndBound) {
    Str8 s("abc");
    EXPECT_EQ(s.size(), 3u);
    EXPECT_STREQ(s.c_str(), "abc");
    EXPECT_FALSE(s.set("123456789"));                // 9 > 8
    EXPECT_STREQ(s.c_str(), "abc");                  // unchanged
    EXPECT_TRUE(s.set("12345678"));
    EXPECT_EQ(s.length(), 8u);
    EXPECT_EQ(s.c_str()[8], '\0');
    EXPECT_FALSE(s.push_back('x'));
    EXPECT_FALSE(s.append("x"));
    EXPECT_TRUE(s.set("ab"));
    EXPECT_EQ(std::strlen(s.get()), 2u);
    EXPECT_TRUE(s.append("cd"));
    EXPECT_TRUE(s.push_back('e'));
    EXPECT_STREQ(s.c_str(), "abcde");
    EXPECT_EQ(s, Str8("abcde"));
    // stale bytes behind the end do not matter for equality
    Str8 t("abcdefgh");
    EXPECT_TRUE(t.set("abcde"));
    EXPECT_EQ(s, t);
    EXPECT_NE(s, Str8("abcd"));
    // truncating constructors
    typedef BoundedAString<4, 3> Str3;
    EXPECT_STREQ(Str3("abcdef").c_str(), "abc");
    EXPECT_STREQ(Str3("abcdef", 2).c_str(), "ab");
    EXPECT_STREQ(Str8(nullptr).c_str(), "");
    s.clear();
    EXPECT_TRUE(s.empty());
    EXPECT_STREQ(s.c_str(), "");
    std::string collected;
    for (char c : Str8("xyz")) { collected += c; }
    EXPECT_EQ(collected, "xyz");
}

TEST(BITIS_BoundedAString, RoundTripAtMaxAndNonAscii) {
    const Str8 s("ABCDEFGH");
    std::vector<uint8_t> bytes;
    ASSERT_EQ(try_serialize(s, bytes), BitisError::Ok);
    EXPECT_EQ(bytes, serialize(BitisAString<4>("ABCDEFGH")));
    Str8 out;
    ASSERT_EQ(try_deserialize_exact(bytes, out), BitisError::Ok);
    EXPECT_EQ(out, s);
    EXPECT_STREQ(out.c_str(), "ABCDEFGH");

    // non-ASCII: encoding is ValueOutOfRange (legacy: masked), decoding InvalidValue
    const Str8 bad("a\xC3");
    EXPECT_EQ(try_serialize(bad, bytes), BitisError::ValueOutOfRange);
    EXPECT_EQ(serialize(bad), serialize(Str8("aC")));
    Bits b; b.add(DynInteger<uint32_t, 32, 4>(2)).add('a', 8).add(0x80, 8);
    EXPECT_EQ(decode(b.bytes(), out), BitisError::InvalidValue);
}

TEST(BITIS_BoundedAString, CountAboveBoundIsInvalidValue) {
    Str8 out;
    Bits nine; nine.add(DynInteger<uint32_t, 32, 4>(9));
    for (int i = 0; i < 9; i++) { nine.add('a', 8); }
    EXPECT_EQ(decode(nine.bytes(), out), BitisError::InvalidValue);
    Bits short_nine; short_nine.add(DynInteger<uint32_t, 32, 4>(9)).add('a', 8);
    EXPECT_EQ(decode(short_nine.bytes(), out), BitisError::InvalidValue);
    Bits short_eight; short_eight.add(DynInteger<uint32_t, 32, 4>(8)).add('a', 8);
    EXPECT_EQ(decode(short_eight.bytes(), out), BitisError::UnexpectedEnd);
}

// ***************************************************************
TEST(BITIS_BoundedDynArray, ApiAndBound) {
    StrList3 l;
    EXPECT_TRUE(l.push_back(Str5("a")));
    EXPECT_TRUE(l.push_back(Str5("bb")));
    EXPECT_TRUE(l.push_back(Str5("ccc")));
    EXPECT_FALSE(l.push_back(Str5("d")));
    EXPECT_EQ(l.size(), 3u);
    EXPECT_STREQ(l.back().c_str(), "ccc");
    EXPECT_FALSE(l.resize(4));
    EXPECT_FALSE(l.set(std::vector<Str5>(4)));
    EXPECT_EQ(l.size(), 3u);
    l.pop_back();
    EXPECT_EQ(l, StrList3({Str5("a"), Str5("bb")}));
    EXPECT_TRUE(l.resize(3));
    EXPECT_EQ(l[2], Str5());                         // reset when shrinking
    EXPECT_EQ(StrList3(std::vector<Str5>(5)).size(), 3u);   // truncating constructor
    std::size_t n = 0;
    for (const auto &s : l) { n += s.size(); }
    EXPECT_EQ(n, 3u);
    l.clear();
    EXPECT_TRUE(l.empty());
}

TEST(BITIS_BoundedDynArray, RoundTripAndBound) {
    const StrList3 l({Str5("abcde"), Str5(""), Str5("xyz")});
    std::vector<uint8_t> bytes;
    ASSERT_EQ(try_serialize(l, bytes), BitisError::Ok);
    StrList3 out;
    ASSERT_EQ(try_deserialize_exact(bytes, out), BitisError::Ok);
    EXPECT_EQ(out, l);

    Bits four; four.add(DynInteger<uint32_t, 32, 2>(4));
    for (int i = 0; i < 4; i++) { four.add(Str5("a")); }
    EXPECT_EQ(decode(four.bytes(), out), BitisError::InvalidValue);
    Bits short_four; short_four.add(DynInteger<uint32_t, 32, 2>(4));
    EXPECT_EQ(decode(short_four.bytes(), out), BitisError::InvalidValue);
    // element bound inside the array
    Bits long_elem; long_elem.add(DynInteger<uint32_t, 32, 2>(1)).add(DynInteger<uint32_t, 32, 4>(6));
    for (int i = 0; i < 6; i++) { long_elem.add('a', 8); }
    EXPECT_EQ(decode(long_elem.bytes(), out), BitisError::InvalidValue);
}

// ***************************************************************
// generated code
static Bounded full_message() {
    Bounded m;
    EXPECT_TRUE(m.name.set("ABCDEFGH"));
    EXPECT_TRUE(m.blob.set(std::vector<uint8_t>{1, 2, 3, 4}));
    EXPECT_TRUE(m.tags.push_back(Str5("aaaaa")));
    EXPECT_TRUE(m.tags.push_back(Str5("bbbbb")));
    EXPECT_TRUE(m.tags.push_back(Str5("ccccc")));
    m.note = Bounded::Note_T(BoundedAString<2, 6>("note!!"));
    m.pair.values[0] = BoundedBinary<2, 3>({7, 8, 9});
    m.pair.values[1] = BoundedBinary<2, 3>({10, 11, 12});
    BLeaf leaf; leaf.v = IntgralWithGivenBitSize<uint16_t, 4>(5);
    EXPECT_TRUE(m.leaves.push_back(leaf));
    EXPECT_TRUE(m.leaves.push_back(leaf));
    for (int i = 0; i < 4; i++) { EXPECT_TRUE(m.colors.push_back(BColor::create_enum<BColorEnum::Blue>())); }
    m.choice.set_oo<OO_Bounded_Choice::OO_Text>(BoundedAString<3, 4>("wxyz"));
    m.tail = BitisBool(true);
    return m;
}

TEST(BITIS_BoundedGenerated, RoundTripAtMax) {
    const Bounded m = full_message();
    std::vector<uint8_t> bytes;
    ASSERT_EQ(try_serialize(m, bytes), BitisError::Ok);
    Bounded out;
    ASSERT_EQ(try_deserialize_exact(bytes, out), BitisError::Ok);
    EXPECT_EQ(out, m);
    EXPECT_STREQ(out.name.c_str(), "ABCDEFGH");
    EXPECT_STREQ(out.choice.get_oo<OO_Bounded_Choice::OO_Text>()->c_str(), "wxyz");
    out.print(-1); printf("\n");

    Bounded other = m;
    other.choice.set_oo<OO_Bounded_Choice::OO_Raw>(BoundedBinary<4, 2>({1, 2}));
    ASSERT_EQ(try_serialize(other, bytes), BitisError::Ok);
    ASSERT_EQ(try_deserialize_exact(bytes, out), BitisError::Ok);
    EXPECT_EQ(out, other);
    EXPECT_NE(out, m);

    // default message
    ASSERT_EQ(try_serialize(Bounded(), bytes), BitisError::Ok);
    ASSERT_EQ(try_deserialize_exact(bytes, out), BitisError::Ok);
    EXPECT_EQ(out, Bounded());
}

TEST(BITIS_BoundedGenerated, CountAboveBoundInMessage) {
    Bounded out;
    // name with count 9 (M = 8): InvalidValue, also if the input ends right after the count
    Bits b; b.add(DynInteger<uint32_t, 32, 4>(9));
    EXPECT_EQ(decode(b.bytes(), out), BitisError::InvalidValue);
    // name ok, blob with count 5 (M = 4)
    Bits b2; b2.add(Str8("ab")).add(DynInteger<uint32_t, 32, 3>(5));
    EXPECT_EQ(decode(b2.bytes(), out), BitisError::InvalidValue);
    // optional note with 7 characters (M = 6)
    Bits b3; b3.add(Str8()).add(Bin4()).add(StrList3()).add(1, 1).add(DynInteger<uint32_t, 32, 2>(7));
    EXPECT_EQ(decode(b3.bytes(), out), BitisError::InvalidValue);
    // the bounded oneof element: text with 5 characters (M = 4)
    Bounded m;
    Bits b4; b4.add(m.name).add(m.blob).add(m.tags).add(m.note).add(m.pair).add(m.leaves).add(m.colors)
        .add(0, 1)                                   // oneof index 0 (dyn bits 1: flag 0)
        .add(DynInteger<uint32_t, 32, 3>(5));
    EXPECT_EQ(decode(b4.bytes(), out), BitisError::InvalidValue);
    // a correct prefix with an M-sized text but nothing more: UnexpectedEnd
    Bits b5; b5.add(m.name).add(m.blob).add(m.tags).add(m.note).add(m.pair).add(m.leaves).add(m.colors)
        .add(0, 1).add(DynInteger<uint32_t, 32, 3>(4));
    EXPECT_EQ(decode(b5.bytes(), out), BitisError::UnexpectedEnd);
}

TEST(BITIS_BoundedGenerated, SetBeyondBoundKeepsValue) {
    Bounded m = full_message();
    EXPECT_FALSE(m.name.set("123456789"));
    EXPECT_FALSE(m.tags.push_back(Str5("d")));
    EXPECT_FALSE(m.leaves.push_back(BLeaf()));
    EXPECT_FALSE(m.colors.push_back(BColor()));
    EXPECT_EQ(m, full_message());
}

// random and truncated input must never crash or read out of bounds (run with ASan)
TEST(BITIS_BoundedGenerated, FuzzDecode) {
    Rng rng(0xB0D5);
    std::vector<uint8_t> valid;
    ASSERT_EQ(try_serialize(full_message(), valid), BitisError::Ok);
    std::size_t decoded = 0;
    for (int i = 0; i < 20000; i++) {
        std::vector<uint8_t> buf;
        if (i % 2 == 0) {
            buf.resize(rng.below(40));
            for (auto &b : buf) { b = static_cast<uint8_t>(rng.next()); }
        } else {
            buf = valid;
            buf.resize(rng.below(static_cast<uint32_t>(valid.size() + 1)));
            if (!buf.empty()) { buf[rng.below(static_cast<uint32_t>(buf.size()))] ^= static_cast<uint8_t>(1u << rng.below(8)); }
        }
        Bounded out;
        if (try_deserialize(buf, out) == BitisError::Ok) {
            decoded++;
            EXPECT_LE(out.name.size(), 8u);
            EXPECT_LE(out.tags.size(), 3u);
            EXPECT_EQ(std::strlen(out.name.c_str()), out.name.size());
            std::vector<uint8_t> re;
            EXPECT_EQ(try_serialize(out, re), BitisError::Ok);
        }
    }
    EXPECT_GT(decoded, 0u);
}

}  // namespace test_bounds
