// Golden vector tests of the C++ runtime (see golden/README.md).
//
// Reads golden/vectors/manifest.txt (path given by BITIS_GOLDEN_MANIFEST) and checks
// - valid:    try_deserialize / try_deserialize_exact consume total_bits, re-encoding gives the
//             same bytes; with appended bytes the default decoder still succeeds, the exact one
//             fails with TrailingData,
// - noncanon: decoding succeeds with total_bits, re-encoding gives the canonical bytes,
// - invalid:  decoding fails with the given BitisError,
// - min_bits / max_bits: bitis_min_bits<T>() / bitis_max_bits<T>() of every message type,
// plus spot checks of decoded values and of the encoder.
//
// The code under test is golden/gen/cpp/golden.h, generated from golden/schema/golden.bitis.
#include <gtest/gtest.h>

#include <cmath>
#include <fstream>
#include <map>
#include <sstream>
#include <string>
#include <vector>

#include "golden.h"

#ifndef BITIS_GOLDEN_MANIFEST
#error "BITIS_GOLDEN_MANIFEST (path of golden/vectors/manifest.txt) has to be defined"
#endif

namespace golden_test {

// ***************************************************************
// manifest

struct Record {
    std::string kind;       // valid, noncanon, invalid, min_bits, max_bits
    std::string name;       // case name (min_bits: the type)
    std::string type;
    std::size_t bits = 0;   // valid/noncanon: total_bits, min_bits: min_bits
    std::vector<uint8_t> data;
    std::vector<uint8_t> canonical;
    std::string error;
    bool exact = false;
};

inline std::vector<uint8_t> unhex(const std::string &s) {
    std::vector<uint8_t> v;
    if (s == "-") { return v; }
    for (std::size_t i = 0; i + 1 < s.size(); i += 2) {
        v.push_back(static_cast<uint8_t>(std::stoul(s.substr(i, 2), nullptr, 16)));
    }
    return v;
}
inline std::string hex(const std::vector<uint8_t> &v) {
    if (v.empty()) { return "-"; }
    std::string s; char b[3];
    for (auto c : v) { snprintf(b, sizeof(b), "%02x", c); s += b; }
    return s;
}

inline std::vector<Record> load_manifest() {
    std::vector<Record> recs;
    std::ifstream in(BITIS_GOLDEN_MANIFEST);
    if (!in) { ADD_FAILURE() << "cannot read " << BITIS_GOLDEN_MANIFEST; return recs; }
    std::string line;
    while (std::getline(in, line)) {
        if (line.empty() || line[0] == '#') { continue; }
        std::istringstream ls(line);
        std::vector<std::string> f;
        std::string w;
        while (ls >> w) { f.push_back(w); }
        Record r;
        r.kind = f[0];
        if (r.kind == "valid" && f.size() == 5) {
            r.name = f[1]; r.type = f[2]; r.bits = std::stoul(f[3]); r.data = unhex(f[4]);
        }
        else if (r.kind == "noncanon" && f.size() == 6) {
            r.name = f[1]; r.type = f[2]; r.bits = std::stoul(f[3]); r.data = unhex(f[4]); r.canonical = unhex(f[5]);
        }
        else if (r.kind == "invalid" && f.size() == 6) {
            r.name = f[1]; r.type = f[2]; r.error = f[3]; r.exact = f[4] == "exact"; r.data = unhex(f[5]);
            if (f[4] != "exact" && f[4] != "default") { ADD_FAILURE() << "bad mode: " << line; }
        }
        else if (r.kind == "min_bits" && f.size() == 3) {
            r.name = f[1]; r.type = f[1]; r.bits = std::stoul(f[2]);
        }
        else if (r.kind == "max_bits" && f.size() == 3) {
            r.name = f[1]; r.type = f[1];
            r.bits = (f[2] == "unbounded") ? BITIS_UNBOUNDED : std::stoul(f[2]);
        }
        else { ADD_FAILURE() << "malformed manifest line: " << line; continue; }
        recs.push_back(r);
    }
    return recs;
}
inline const std::vector<Record> &manifest() {
    static const std::vector<Record> recs = load_manifest();
    return recs;
}
inline std::vector<Record> records(const std::string &kind) {
    std::vector<Record> r;
    for (const auto &rec : manifest()) { if (rec.kind == kind) { r.push_back(rec); } }
    return r;
}
inline std::vector<uint8_t> case_bytes(const std::string &name) {
    for (const auto &rec : manifest()) {
        if ((rec.kind == "valid" || rec.kind == "noncanon") && rec.name == name) { return rec.data; }
    }
    ADD_FAILURE() << "no case " << name;
    return {};
}

// ***************************************************************
// decoding by type name

struct Outcome {
    BitisError error = BitisError::Ok;
    std::vector<uint8_t> reencoded;
    std::size_t bits = 0;
};

template<typename T>
Outcome roundtrip(const std::vector<uint8_t> &data, const bool exact) {
    Outcome o;
    T v;
    BitisSize size{0, 0};
    o.error = exact ? try_deserialize_exact<T>(data, v, &size) : try_deserialize<T>(data, v, &size);
    if (o.error != BitisError::Ok) { return o; }
    o.bits = size.bits;
    EXPECT_EQ(size.bytes, (size.bits + 7) / 8);
    EXPECT_EQ(try_serialize(v, o.reencoded), BitisError::Ok) << "re-encoding a decoded value must not fail";
    return o;
}

typedef Outcome (*RoundtripFn)(const std::vector<uint8_t> &, bool);
typedef std::size_t (*SizeFn)();
struct TypeEntry { RoundtripFn roundtrip; SizeFn min_bits; SizeFn max_bits; };

template<typename T> std::size_t min_bits_of() { return bitis_min_bits<T>(); }
template<typename T> std::size_t max_bits_of() { return bitis_max_bits<T>(); }
template<typename T> TypeEntry entry() { return TypeEntry{&roundtrip<T>, &min_bits_of<T>, &max_bits_of<T>}; }

inline const std::map<std::string, TypeEntry> &types() {
    static const std::map<std::string, TypeEntry> t = {
        {"Empty", entry<Empty>()}, {"Ints", entry<Ints>()}, {"DynInts", entry<DynInts>()},
        {"Floats", entry<Floats>()}, {"FixedPoints", entry<FixedPoints>()}, {"Texts", entry<Texts>()},
        {"Leaf", entry<Leaf>()}, {"Mid", entry<Mid>()}, {"Nested", entry<Nested>()},
        {"Containers", entry<Containers>()}, {"Enums", entry<Enums>()}, {"IntList", entry<IntList>()},
        {"Choice", entry<Choice>()}, {"Everything", entry<Everything>()},
        {"BoundsItem", entry<BoundsItem>()}, {"Bounds", entry<Bounds>()},
    };
    return t;
}
inline Outcome decode_by_name(const std::string &type, const std::vector<uint8_t> &data, const bool exact) {
    const auto it = types().find(type);
    if (it == types().end()) { ADD_FAILURE() << "unknown message type " << type; return Outcome{BitisError::Ok, {}, 0}; }
    return it->second.roundtrip(data, exact);
}

// ***************************************************************
// one gtest per manifest record

std::string record_name(const testing::TestParamInfo<Record> &info) { return info.param.name; }
void PrintTo(const Record &r, std::ostream *os) { *os << r.kind << " " << r.name; }

class GoldenValid : public testing::TestWithParam<Record> { };
TEST_P(GoldenValid, DecodeReencode) {
    const Record &r = GetParam();
    ASSERT_EQ(r.data.size(), (r.bits + 7) / 8) << "hex length / total_bits mismatch in the manifest";
    for (const bool exact : {false, true}) {
        const auto o = decode_by_name(r.type, r.data, exact);
        ASSERT_EQ(o.error, BitisError::Ok) << "exact=" << exact << ": " << bitis_error_name(o.error);
        EXPECT_EQ(o.bits, r.bits) << "exact=" << exact;
        EXPECT_EQ(hex(o.reencoded), hex(r.data)) << "exact=" << exact;
    }
    // trailing bytes: ignored by the default decoder, rejected by the exact one
    auto longer = r.data;
    longer.push_back(0xa5); longer.push_back(0xff);
    const auto o = decode_by_name(r.type, longer, false);
    ASSERT_EQ(o.error, BitisError::Ok) << bitis_error_name(o.error);
    EXPECT_EQ(o.bits, r.bits);
    EXPECT_EQ(hex(o.reencoded), hex(r.data));
    EXPECT_EQ(decode_by_name(r.type, longer, true).error, BitisError::TrailingData);
}
INSTANTIATE_TEST_SUITE_P(Golden, GoldenValid, testing::ValuesIn(records("valid")), record_name);

class GoldenNonCanon : public testing::TestWithParam<Record> { };
TEST_P(GoldenNonCanon, DecodeReencode) {
    const Record &r = GetParam();
    for (const bool exact : {false, true}) {
        const auto o = decode_by_name(r.type, r.data, exact);
        ASSERT_EQ(o.error, BitisError::Ok) << "exact=" << exact << ": " << bitis_error_name(o.error);
        EXPECT_EQ(o.bits, r.bits) << "exact=" << exact;
        EXPECT_EQ(hex(o.reencoded), hex(r.canonical)) << "exact=" << exact;
    }
}
INSTANTIATE_TEST_SUITE_P(Golden, GoldenNonCanon, testing::ValuesIn(records("noncanon")), record_name);

class GoldenInvalid : public testing::TestWithParam<Record> { };
TEST_P(GoldenInvalid, Error) {
    const Record &r = GetParam();
    const auto o = decode_by_name(r.type, r.data, r.exact);
    EXPECT_STREQ(bitis_error_name(o.error), r.error.c_str())
        << (o.error == BitisError::Ok ? "decoded, re-encoded: " + hex(o.reencoded) : std::string());
}
INSTANTIATE_TEST_SUITE_P(Golden, GoldenInvalid, testing::ValuesIn(records("invalid")), record_name);

class GoldenMinBits : public testing::TestWithParam<Record> { };
TEST_P(GoldenMinBits, MinBits) {
    const Record &r = GetParam();
    const auto it = types().find(r.type);
    ASSERT_NE(it, types().end()) << r.type;
    EXPECT_EQ(it->second.min_bits(), r.bits);
}
INSTANTIATE_TEST_SUITE_P(Golden, GoldenMinBits, testing::ValuesIn(records("min_bits")), record_name);

class GoldenMaxBits : public testing::TestWithParam<Record> { };
TEST_P(GoldenMaxBits, MaxBits) {
    const Record &r = GetParam();
    const auto it = types().find(r.type);
    ASSERT_NE(it, types().end()) << r.type;
    EXPECT_EQ(it->second.max_bits(), r.bits);
    // every valid encoding of the type stays within the bound
    for (const auto &v : records("valid")) {
        if (v.type == r.type) { EXPECT_LE(v.bits, r.bits) << v.name; }
    }
}
INSTANTIATE_TEST_SUITE_P(Golden, GoldenMaxBits, testing::ValuesIn(records("max_bits")), record_name);

TEST(Golden, ManifestMatchesSchema) {
    char stamp[64];
    snprintf(stamp, sizeof(stamp), "schema hash %016" PRIx64, bitis_schema_golden::BITIS_SCHEMA_HASH);
    std::ifstream in(BITIS_GOLDEN_MANIFEST);
    std::stringstream ss; ss << in.rdbuf();
    EXPECT_NE(ss.str().find(stamp), std::string::npos) << "manifest generated for another schema, expected " << stamp;
    EXPECT_GT(records("valid").size(), 40u);
    EXPECT_GT(records("invalid").size(), 40u);
}

// ***************************************************************
// spot checks: decoded values

template<typename T>
T decode(const std::string &name) {
    T v;
    const auto bytes = case_bytes(name);
    EXPECT_EQ(try_deserialize_exact(bytes, v), BitisError::Ok) << name;
    return v;
}

TEST(GoldenSpotCheck, Ints) {
    auto m = decode<Ints>("ints_min");
    EXPECT_EQ(m.val_i1.value, -1);
    EXPECT_EQ(m.val_i7.value, -127);
    EXPECT_EQ(m.val_i16.value, -32767);
    EXPECT_EQ(m.val_i32.value, -2147483647);
    EXPECT_EQ(m.val_i33.value, -8589934591LL);
    EXPECT_EQ(m.val_i64.value, -INT64_MAX);
    m = decode<Ints>("ints_max");
    EXPECT_EQ(m.val_u33.value, (1ULL << 33) - 1);
    EXPECT_EQ(m.val_u63.value, UINT64_MAX >> 1);
    EXPECT_EQ(m.val_u64.value, UINT64_MAX);
    EXPECT_EQ(m.val_i64.value, INT64_MAX);
    m = decode<Ints>("noncanon_ints_negzero");
    EXPECT_EQ(m.val_i1.value, 0);
    EXPECT_EQ(m.val_i16.value, 0);
    EXPECT_EQ(m.val_i64.value, 0);
}

TEST(GoldenSpotCheck, DynInts) {
    auto m = decode<DynInts>("dynints_min");
    EXPECT_EQ(m.val_i16d5.value, -32767);
    EXPECT_EQ(m.val_i64d8.value, -INT64_MAX);
    m = decode<DynInts>("dynints_max");
    EXPECT_EQ(m.val_u8d3.value, 255);
    EXPECT_EQ(m.val_u64d7.value, UINT64_MAX);
    m = decode<DynInts>("noncanon_dynints_nonminimal");
    EXPECT_EQ(m.val_u8d3.value, 5);
    EXPECT_EQ(m.val_u32d4.value, 5u);
    EXPECT_EQ(m.val_i16d5.value, -3);
}

TEST(GoldenSpotCheck, Floats) {
    auto m = decode<Floats>("floats_negzero");
    EXPECT_TRUE(m.val_f64.value == 0.0 && std::signbit(m.val_f64.value) && std::signbit(m.val_f32.value));
    m = decode<Floats>("floats_nan");
    EXPECT_TRUE(std::isnan(m.val_f32.value) && std::isnan(m.val_f64.value));
    m = decode<Floats>("floats_inf");
    EXPECT_TRUE(std::isinf(m.val_f32.value) && m.val_f32.value > 0 && std::isinf(m.val_f64.value) && m.val_f64.value < 0);
    m = decode<Floats>("floats_subnormal");
    EXPECT_EQ(bitis_helper::to_u64_bits(m.val_f32.value), 1u);
    EXPECT_EQ(bitis_helper::to_u64_bits(m.val_f64.value), 1u);
}

TEST(GoldenSpotCheck, FixedPoints) {
    auto m = decode<FixedPoints>("fixedpoints_gnss");
    ASSERT_TRUE(m.lat.has_value() && m.lng.has_value() && m.fp2.has_value() && m.fp53.has_value());
    EXPECT_NEAR(m.lat.get_value(), 51.756349, 6e-9);
    EXPECT_NEAR(m.lng.get_value(), 14.332868, 6e-9);
    // 0.49999999999999994 + 1.0 is 1.5 in f64 (spec 2.7), rounded away from zero
    EXPECT_EQ(m.fp2.get_value(), 1.0);
    EXPECT_NEAR(m.fp10.get_value(), -1.0, 0.0025);
    EXPECT_NEAR(m.fp53.get_value(), 123456.789, 1e-9);
    m = decode<FixedPoints>("fixedpoints_gnss_neg");
    EXPECT_NEAR(m.lat.get_value(), -33.856784, 6e-9);
    EXPECT_NEAR(m.lng.get_value(), -151.215297, 6e-9);
    m = decode<FixedPoints>("fixedpoints_min");
    EXPECT_EQ(m.fp2.get_value(), 0.0); EXPECT_EQ(m.fp10.get_value(), -2.0);
    EXPECT_EQ(m.lat.get_value(), -90.0); EXPECT_EQ(m.lng.get_value(), -180.0); EXPECT_EQ(m.fp53.get_value(), -1e6);
    m = decode<FixedPoints>("fixedpoints_max");
    EXPECT_EQ(m.fp2.get_value(), 1.0); EXPECT_EQ(m.fp10.get_value(), 3.0);
    EXPECT_EQ(m.lat.get_value(), 90.0); EXPECT_EQ(m.lng.get_value(), 180.0); EXPECT_EQ(m.fp53.get_value(), 1e6);
    m = decode<FixedPoints>("fixedpoints_default");
    EXPECT_TRUE(m.fp2.is_overflow() && m.fp10.is_overflow() && m.lat.is_overflow() && m.lng.is_overflow()
        && m.fp53.is_overflow());
    m = decode<FixedPoints>("fixedpoints_underflow");
    EXPECT_TRUE(m.fp2.is_underflow() && m.fp10.is_underflow() && m.lat.is_underflow() && m.lng.is_underflow()
        && m.fp53.is_underflow());
    m = decode<FixedPoints>("fixedpoints_zero");
    EXPECT_EQ(m.fp2.get_value(), 1.0);   // 0.5 is a rounding tie (t = 1.5)
}

TEST(GoldenSpotCheck, Texts) {
    auto m = decode<Texts>("texts_values");
    EXPECT_STREQ(m.text.get(), "Hello, bitis!");
    EXPECT_EQ(m.data.get(), (std::vector<uint8_t>{0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff}));
    m = decode<Texts>("texts_ascii_edges");
    ASSERT_EQ(m.text.size(), 4u);
    EXPECT_EQ(std::string(m.text.get(), m.text.size()), std::string("\0 ~\x7f", 4));
    m = decode<Texts>("texts_long");
    EXPECT_EQ(m.text.size(), 43u);
    ASSERT_EQ(m.data.size(), 300u);
    EXPECT_EQ(m.data.get()[299], 43);
}

TEST(GoldenSpotCheck, ContainersEnumsChoiceNested) {
    auto c = decode<Containers>("containers_full");
    ASSERT_FALSE(c.opt_color.is_none);
    EXPECT_TRUE(c.opt_color.value.is_enum<ColorEnum::Yellow>());
    ASSERT_FALSE(c.opt_text.is_none);
    EXPECT_EQ(c.opt_text.value.size(), 0u);
    EXPECT_EQ(c.fixed_ints.values[0].value, -127);
    EXPECT_EQ(c.fixed_ints.values[2].value, 127);
    ASSERT_EQ(c.dyn_leaves.values.size(), 3u);
    EXPECT_EQ(c.dyn_leaves.values[2].val.value, -63);
    EXPECT_STREQ(c.dyn_leaves.values[0].tag.get(), "deep");
    c = decode<Containers>("containers_default");
    EXPECT_TRUE(c.opt_u7.is_none && c.opt_leaf.is_none && c.dyn_ints.values.empty());

    auto e = decode<Enums>("enums_all");
    EXPECT_TRUE(e.color.is_enum<ColorEnum::Yellow>());
    EXPECT_TRUE(e.level.is_enum<LevelEnum::High>());
    ASSERT_EQ(e.colors.values.size(), 6u);
    EXPECT_TRUE(e.colors.values[5].is_enum<ColorEnum::Yellow>());
    e = decode<Enums>("enums_default");
    EXPECT_TRUE(e.color.is_enum<ColorEnum::Green>() && e.level.is_enum<LevelEnum::Low>());

    typedef OO_Choice_Value CV;
    auto ch = decode<Choice>("choice_text");
    ASSERT_TRUE(ch.value.is_oo_value<CV::OO_Text>());
    EXPECT_STREQ(ch.value.get_oo<CV::OO_Text>()->get(), "oneof text");
    EXPECT_TRUE(ch.tail.value);
    ch = decode<Choice>("choice_big");
    ASSERT_TRUE(ch.value.is_oo_value<CV::OO_Big>());
    EXPECT_EQ(ch.value.get_oo<CV::OO_Big>()->value, -2147483647);
    ch = decode<Choice>("choice_list");
    ASSERT_TRUE(ch.value.is_oo_value<CV::OO_List>());
    const auto &items = ch.value.get_oo<CV::OO_List>()->items.values;
    ASSERT_EQ(items.size(), 5u);
    EXPECT_EQ(items[0].value, -1); EXPECT_EQ(items[3].value, 127); EXPECT_EQ(items[4].value, -127);
    ch = decode<Choice>("choice_nothing");
    EXPECT_TRUE(ch.value.is_oo_value<CV::OO_Nothing>() && ch.tail.value);

    auto n = decode<Nested>("nested_deep");
    EXPECT_EQ(n.mid.id.value, 99);
    EXPECT_EQ(n.mid.leaf.val.value, -42);
    EXPECT_STREQ(n.mid.leaf.tag.get(), "level3");
    ASSERT_FALSE(n.mid.extra.is_none);
    EXPECT_STREQ(n.mid.extra.value.tag.get(), "extra");
    EXPECT_TRUE(n.after.value);

    auto all = decode<Everything>("everything_full");
    EXPECT_EQ(all.tail.value, 5);
    EXPECT_EQ(all.ints.val_i64.value, INT64_MAX);
    EXPECT_EQ(all.nested.mid.leaf.val.value, -42);
}

// ***************************************************************
// spot checks: encoder (values set from physical / native values)

template<typename T>
std::string enc(const T &v) {
    std::vector<uint8_t> out;
    EXPECT_EQ(try_serialize(v, out), BitisError::Ok);
    return hex(out);
}

TEST(GoldenSpotCheck, Encoder) {
    // fixed point quantization incl. exact rounding ties (round half away from zero)
    FixedPoints ties;
    ties.fp2 = FixedPoints::Fp2_T(0.5);
    ties.fp10 = FixedPoints::Fp10_T(-1.845739471106758);
    ties.lat = FixedPoints::Lat_T(-1.5684636309743063e-05);
    ties.lng = FixedPoints::Lng_T(-1.5684636309743063e-05);
    ties.fp53 = FixedPoints::Fp53_T(-3.32365743815901e-07);
    EXPECT_EQ(enc(ties), hex(case_bytes("fixedpoints_ties")));

    FixedPoints gnss;
    gnss.fp2 = FixedPoints::Fp2_T(0.49999999999999994);
    gnss.fp10 = FixedPoints::Fp10_T(-1.0);
    gnss.lat = FixedPoints::Lat_T(51.756349);
    gnss.lng = FixedPoints::Lng_T(14.332868);
    gnss.fp53 = FixedPoints::Fp53_T(123456.789);
    EXPECT_EQ(enc(gnss), hex(case_bytes("fixedpoints_gnss")));

    FixedPoints under;
    under.fp2 = FixedPoints::Fp2_T(NAN);
    under.fp10 = FixedPoints::Fp10_T(-2.001);
    under.lat = FixedPoints::Lat_T(-90.000001);
    under.lng = FixedPoints::Lng_T(-INFINITY);
    under.fp53 = FixedPoints::Fp53_T(-1000000.001);
    EXPECT_EQ(enc(under), hex(case_bytes("fixedpoints_underflow")));
    EXPECT_EQ(enc(FixedPoints()), hex(case_bytes("fixedpoints_default")));

    Ints ints;
    ints.val_i1 = Ints::ValI1_T(-1);
    ints.val_i7 = Ints::ValI7_T(-127);
    ints.val_i16 = Ints::ValI16_T(-32767);
    ints.val_i32 = Ints::ValI32_T(-2147483647);
    ints.val_i33 = Ints::ValI33_T(-8589934591LL);
    ints.val_i64 = Ints::ValI64_T(-INT64_MAX);
    EXPECT_EQ(enc(ints), hex(case_bytes("ints_min")));

    Texts texts;
    texts.text = Texts::Text_T("Hello, bitis!");
    texts.data = Texts::Data_T(std::vector<uint8_t>{0x00, 0x01, 0x7f, 0x80, 0xfe, 0xff});
    EXPECT_EQ(enc(texts), hex(case_bytes("texts_values")));

    EXPECT_EQ(enc(Empty()), "-");
    EXPECT_EQ(enc(Everything()), hex(case_bytes("everything_default")));
    EXPECT_EQ(enc(Choice()), hex(case_bytes("choice_default")));

    // values outside the valid range are rejected by the checked encoder
    std::vector<uint8_t> out;
    Ints bad;
    bad.val_i64 = Ints::ValI64_T(INT64_MIN);
    EXPECT_EQ(try_serialize(bad, out), BitisError::ValueOutOfRange);
    bad = Ints();
    bad.val_i16 = Ints::ValI16_T(INT16_MIN);
    EXPECT_EQ(try_serialize(bad, out), BitisError::ValueOutOfRange);
    bad = Ints();
    bad.val_u7 = Ints::ValU7_T(128);
    EXPECT_EQ(try_serialize(bad, out), BitisError::ValueOutOfRange);
    DynInts bad_dyn;
    bad_dyn.val_i16d5 = DynInts::ValI16D5_T(INT16_MIN);
    EXPECT_EQ(try_serialize(bad_dyn, out), BitisError::ValueOutOfRange);
    Texts bad_text;
    bad_text.text = Texts::Text_T("\xc3\xa4");
    EXPECT_EQ(try_serialize(bad_text, out), BitisError::ValueOutOfRange);
}

// maximum lengths (spec 2.16): heap-free containers that cannot exceed M
TEST(GoldenSpotCheck, Bounds) {
    auto m = decode<Bounds>("bounds_max");
    EXPECT_STREQ(m.name.c_str(), "12345678");
    EXPECT_EQ(m.blob, Bounds::Blob_T({0x00, 0x80, 0xff, 0x7f}));
    ASSERT_EQ(m.tags.size(), 3u);
    EXPECT_STREQ(m.tags[2].c_str(), "ef");
    ASSERT_FALSE(m.note.is_none);
    EXPECT_STREQ(m.note.value.c_str(), "note!");
    ASSERT_EQ(m.items.size(), 2u);
    EXPECT_EQ(m.items[1].val.value, 9);
    ASSERT_FALSE(m.items[0].label.is_none);
    EXPECT_STREQ(m.items[0].label.value.c_str(), "xyz");
    m = decode<Bounds>("bounds_mixed");
    EXPECT_STREQ(m.name.c_str(), "x");
    EXPECT_TRUE(m.note.is_none);
    EXPECT_EQ(m.tags.size(), 1u);

    // encoder: values at M give the vector; the containers refuse to grow beyond M
    Bounds b;
    EXPECT_TRUE(b.name.set("12345678"));
    EXPECT_FALSE(b.name.set("123456789"));
    EXPECT_FALSE(b.name.push_back('9'));
    EXPECT_STREQ(b.name.c_str(), "12345678");
    const uint8_t blob[] = {0x00, 0x80, 0xff, 0x7f, 0x01};
    EXPECT_FALSE(b.blob.set(blob, 5));
    EXPECT_TRUE(b.blob.set(blob, 4));
    EXPECT_FALSE(b.blob.push_back(1));
    for (const char *t : {"ab", "cd", "ef"}) { EXPECT_TRUE(b.tags.push_back(Bounds::Tags_T::ValT(t))); }
    EXPECT_FALSE(b.tags.push_back(Bounds::Tags_T::ValT("gh")));
    EXPECT_FALSE(b.tags[0].set("abc"));
    b.note = Bounds::Note_T(BoundedAString<4, 5>("note!"));
    BoundsItem item;
    item.val = BoundsItem::Val_T(15);
    item.label = BoundsItem::Label_T(BoundedAString<2, 3>("xyz"));
    EXPECT_TRUE(b.items.push_back(item));
    item.val = BoundsItem::Val_T(9);
    item.label = BoundsItem::Label_T(BoundedAString<2, 3>("~~~"));
    EXPECT_TRUE(b.items.push_back(item));
    EXPECT_FALSE(b.items.push_back(item));
    EXPECT_EQ(enc(b), hex(case_bytes("bounds_max")));
    EXPECT_EQ(b, decode<Bounds>("bounds_max"));
    EXPECT_EQ(enc(Bounds()), hex(case_bytes("bounds_default")));
    // min_bits / max_bits of the bounded containers (spec section 4)
    static_assert(Bounds::Name_T::max_bits() == 6 + 64, "astr_d4[8]");
    static_assert(Bounds::Blob_T::max_bits() == 5 + 32, "binary_d3[4]");
    static_assert(Bounds::Tags_T::max_bits() == 4 + 3 * (4 + 16), "repeated_dyn_2[3] astr_d2[2]");
    static_assert(bitis_min_bits<Bounds>() == 5, "min_bits");
}

// max_bits (docs/WIRE_FORMAT.md section 4), computed by hand from the schema
TEST(GoldenMaxBits, HandComputed) {
    // fixed size: max = min
    EXPECT_EQ(bitis_max_bits<Ints>(), 360u);
    EXPECT_EQ(bitis_max_bits<Floats>(), 96u);
    EXPECT_EQ(bitis_max_bits<FixedPoints>(), 2u + 10u + 34u + 35u + 53u);
    EXPECT_EQ(bitis_max_bits<Empty>(), 0u);
    // dynmax(X, Y) = X + ceil(X / Y), signed + 1: u8d3 11, u32d4 40, u64d64 65, u64d7 74, i16d5 21, i64d8 73
    EXPECT_EQ(bitis_max_bits<DynInts>(), 11u + 40u + 65u + 74u + 21u + 73u);
    EXPECT_EQ(bitis_max_bytes<DynInts>(), 36u);
    // enums: dyn(k - 1, 32, Y); Color 6 values Y=2: 5 -> 1+2+1+2+1, Level 3 values Y=1: 2 -> 1+1+1+1+1
    EXPECT_EQ(bitis_max_bits<Color>(), 7u);
    EXPECT_EQ(bitis_max_bits<Level>(), 5u);
    // astr / binary / repeated_dyn without a maximum length are unbounded, and so is everything containing them
    for (const std::size_t b : {bitis_max_bits<Texts>(), bitis_max_bits<Leaf>(), bitis_max_bits<Mid>(),
                                bitis_max_bits<Nested>(), bitis_max_bits<Containers>(), bitis_max_bits<Enums>(),
                                bitis_max_bits<IntList>(), bitis_max_bits<Choice>(), bitis_max_bits<Everything>()}) {
        EXPECT_EQ(b, BITIS_UNBOUNDED);
    }
    EXPECT_EQ(bitis_max_bytes<Everything>(), BITIS_UNBOUNDED);
    // never below min_bits; the encodings of the valid vectors stay within the bound
    for (const auto &r : records("valid")) {
        const auto it = types().find(r.type);
        ASSERT_NE(it, types().end());
        if (r.type == "Ints") { EXPECT_LE(r.bits, bitis_max_bits<Ints>()); }
        if (r.type == "DynInts") { EXPECT_LE(r.bits, bitis_max_bits<DynInts>()); }
        if (r.type == "FixedPoints") { EXPECT_EQ(r.bits, bitis_max_bits<FixedPoints>()); }
    }
}

}
