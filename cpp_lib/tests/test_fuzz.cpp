// Deterministic fuzzing of the decoder with random and mutated buffers (run the test
// binary with -DBITIS_SANITIZE=ON to check for out-of-bounds reads, leaks and UB).
// Property: decoding never crashes; anything that decodes re-encodes without error
// and re-encoding is stable (encode(decode(encode(x))) == encode(x)).
#include <gtest/gtest.h>
#include <map>

#include "bitis_lib.h"
#include "test_helpers.h"

using namespace bitis_test;

namespace test_fuzz {

struct Stats {
    std::size_t total = 0;
    std::size_t ok = 0;
    std::map<BitisError, std::size_t> errors;
};

template<typename T>
void decode_one(const std::vector<uint8_t> &buf, Stats &st) {
    st.total++;
    // exact size heap copy so that ASan detects any read past the end
    const std::size_t len = buf.size();
    std::unique_ptr<uint8_t[]> data(new uint8_t[len ? len : 1]);
    if (len) { std::memcpy(data.get(), buf.data(), len); }

    T m;
    BitisSize size{0, 0};
    const auto e = try_deserialize<T>(data.get(), len, m, &size);
    if (e == BitisError::Ok) {
        st.ok++;
        ASSERT_LE(size.bytes, len);
        ASSERT_EQ(size.bytes, (size.bits + 7) / 8);
        std::vector<uint8_t> re;
        ASSERT_EQ(try_serialize(m, re), BitisError::Ok);
        ASSERT_LE(re.size(), size.bytes);   // non-minimal dyn integers are accepted
        T m2;
        ASSERT_EQ(try_deserialize_exact(re, m2), BitisError::Ok);
        ASSERT_EQ(serialize(m2), re);
        // the strict decoder only differs by TrailingData
        T m3;
        const auto ex = try_deserialize_exact<T>(data.get(), len, m3);
        ASSERT_EQ(ex, size.bytes == len ? BitisError::Ok : BitisError::TrailingData);
    } else {
        st.errors[e]++;
        ASSERT_NE(e, BitisError::TrailingData);
        ASSERT_NE(e, BitisError::ValueOutOfRange);
        // legacy API must not crash either
        const auto legacy = deserialize_with_size<T>(std::vector<uint8_t>(data.get(), data.get() + len));
        ASSERT_EQ(legacy.bits, 0u);
    }
}

template<typename T>
void report(const char *name, const Stats &st) {
    printf("fuzz %-12s: %zu buffers, %zu decoded, UnexpectedEnd %zu, InvalidValue %zu, InvalidDiscriminant %zu\n",
        name, st.total, st.ok,
        st.errors.count(BitisError::UnexpectedEnd) ? st.errors.at(BitisError::UnexpectedEnd) : 0,
        st.errors.count(BitisError::InvalidValue) ? st.errors.at(BitisError::InvalidValue) : 0,
        st.errors.count(BitisError::InvalidDiscriminant) ? st.errors.at(BitisError::InvalidDiscriminant) : 0);
}

template<typename T>
void fuzz_type(const char *name, const std::vector<std::vector<uint8_t>> &seeds, uint64_t rng_seed,
               std::size_t n_random, std::size_t n_mutated) {
    Rng rng(rng_seed);
    Stats st;
    // purely random buffers
    for (std::size_t i = 0; i < n_random && !::testing::Test::HasFatalFailure(); i++) {
        std::vector<uint8_t> buf(rng.below(48));
        for (auto &b : buf) { b = static_cast<uint8_t>(rng.next() >> 56); }
        decode_one<T>(buf, st);
    }
    // mutated valid encodings (bit flips, truncation, extension)
    for (std::size_t i = 0; i < n_mutated && !::testing::Test::HasFatalFailure(); i++) {
        std::vector<uint8_t> buf = seeds[rng.below(static_cast<uint32_t>(seeds.size()))];
        const uint32_t flips = 1 + rng.below(4);
        for (uint32_t f = 0; f < flips && !buf.empty(); f++) {
            const uint32_t bit = rng.below(static_cast<uint32_t>(buf.size() * 8));
            buf[bit / 8] = static_cast<uint8_t>(buf[bit / 8] ^ (1u << (bit % 8)));
        }
        const uint32_t action = rng.below(8);
        if (action == 0 && !buf.empty()) { buf.resize(rng.below(static_cast<uint32_t>(buf.size()))); }
        else if (action == 1) { for (uint32_t k = rng.below(8); k > 0; k--) buf.push_back(static_cast<uint8_t>(rng.next())); }
        decode_one<T>(buf, st);
    }
    report<T>(name, st);
    EXPECT_GT(st.ok, 0u);
}

TEST(BITIS_Fuzz, RandomAndMutatedBuffers) {
    std::vector<std::vector<uint8_t>> xl_seeds;
    for (const auto &m : xl_vectors()) xl_seeds.push_back(serialize(m));
    std::vector<std::vector<uint8_t>> all_seeds;
    for (int i = 0; i < 5; i++) all_seeds.push_back(serialize(all_types_sample(i)));
    std::vector<std::vector<uint8_t>> kv_seeds{serialize(kv_sample())};
    std::vector<std::vector<uint8_t>> kvoo_seeds;
    for (const auto &e : kv_sample().entries.values) kvoo_seeds.push_back(serialize(e));
    MsgKVMapSimple simple;
    MsgKVSimple s; s.key = MsgKVSimple::Key_T("key"); s.value = MsgKVSimple::Value_T("value");
    simple.entries.values.push_back(s); simple.entries.values.push_back(s);
    std::vector<std::vector<uint8_t>> simple_seeds{serialize(simple)};

    fuzz_type<XL>("XL", xl_seeds, 1, 15000, 15000);
    fuzz_type<AllTypes>("AllTypes", all_seeds, 2, 15000, 15000);
    fuzz_type<MsgKVMapOO>("MsgKVMapOO", kv_seeds, 3, 10000, 10000);
    fuzz_type<MsgKVOO>("MsgKVOO", kvoo_seeds, 4, 10000, 10000);
    fuzz_type<MsgKVMapSimple>("KVMapSimple", simple_seeds, 5, 5000, 5000);
    fuzz_type<OO_Xl_Val>("OO_Xl_Val", {serialize(xl_vectors()[1].val), serialize(xl_vectors()[2].val)}, 6, 5000, 5000);
}

}
