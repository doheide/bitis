// Tests of the CRC-32 helper bitis_crc32.h against the shared vectors
// golden/vectors/crc32.txt (path given by BITIS_GOLDEN_CRC32, see golden/README.md):
// one-shot, incremental with several splits, byte by byte, reset.
//
// Compiled twice into bitis_golden: with the default nibble table (this file) and with
// BITIS_CRC32_TABLE256 (test_crc32_table256.cpp, which includes this file).
#include <gtest/gtest.h>

#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "bitis_crc32.h"

#ifndef BITIS_GOLDEN_CRC32
#error "BITIS_GOLDEN_CRC32 (path of golden/vectors/crc32.txt) has to be defined"
#endif
#ifndef BITIS_CRC32_TEST_SUITE
#define BITIS_CRC32_TEST_SUITE Crc32Nibble16
#endif

namespace {

struct CrcVector {
    std::string name;
    std::vector<uint8_t> data;
    uint32_t crc = 0;
};

std::vector<CrcVector> load_crc_vectors() {
    std::ifstream in(BITIS_GOLDEN_CRC32);
    EXPECT_TRUE(in.good()) << "cannot open " << BITIS_GOLDEN_CRC32;
    std::vector<CrcVector> out;
    std::string line;
    while (std::getline(in, line)) {
        if (line.empty() || line[0] == '#') { continue; }
        std::istringstream ls(line);
        std::string kind, name, input, crc;
        ls >> kind >> name >> input >> crc;
        CrcVector v;
        v.name = name;
        if (kind == "hex") {
            if (input != "-") {
                EXPECT_EQ(input.size() % 2, 0u) << line;
                for (std::size_t i = 0; i + 1 < input.size(); i += 2) {
                    v.data.push_back(static_cast<uint8_t>(std::stoul(input.substr(i, 2), nullptr, 16)));
                }
            }
        } else if (kind == "ascii") {
            v.data.assign(input.begin(), input.end());
        } else {
            ADD_FAILURE() << "unknown record kind: " << line;
            continue;
        }
        EXPECT_EQ(crc.size(), 8u) << line;
        v.crc = static_cast<uint32_t>(std::stoul(crc, nullptr, 16));
        out.push_back(v);
    }
    return out;
}

// constexpr evaluation (both table variants)
constexpr uint8_t kCheck[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
static_assert(bitis_crc32(kCheck, sizeof(kCheck)) == 0xCBF43926u, "check value");
static_assert(bitis_crc32(kCheck, 0) == 0u, "empty input");

} // namespace

TEST(BITIS_CRC32_TEST_SUITE, CheckValue) {
    EXPECT_EQ(bitis_crc32("123456789", 9), 0xCBF43926u);
    EXPECT_EQ(bitis_crc32(kCheck, 0), 0u);
    EXPECT_EQ(BitisCrc32().finish(), 0u);
    const std::string s = "123456789";
    EXPECT_EQ(bitis_crc32(s.data(), s.size()), 0xCBF43926u);
}

TEST(BITIS_CRC32_TEST_SUITE, Vectors) {
    const auto vectors = load_crc_vectors();
    ASSERT_GE(vectors.size(), 16u);
    bool have_check = false;
    for (const auto &v : vectors) {
        SCOPED_TRACE(v.name);
        const uint8_t *d = v.data.data();
        const std::size_t n = v.data.size();
        if (v.name == "check") {
            have_check = true;
            EXPECT_EQ(v.crc, 0xCBF43926u);
        }
        EXPECT_EQ(bitis_crc32(d, n), v.crc);

        // splits at the start, the end, the middle and every position of short inputs
        std::vector<std::size_t> cuts = {0, 1, n / 2, n > 0 ? n - 1 : 0, n};
        if (n <= 64) { for (std::size_t i = 0; i <= n; ++i) { cuts.push_back(i); } }
        for (std::size_t i : cuts) {
            if (i > n) { continue; }
            BitisCrc32 c;
            c.update(d, i);
            c.update(d + i, n - i);
            EXPECT_EQ(c.finish(), v.crc) << "split at " << i;
        }
        // three pieces, with empty updates in between
        {
            const std::size_t a = n / 3, b = 2 * n / 3;
            BitisCrc32 c;
            c.update(d, a);
            c.update(d + a, 0);
            c.update(d + a, b - a);
            c.update(d + b, n - b);
            c.update(d + n, 0);
            EXPECT_EQ(c.finish(), v.crc) << "three pieces";
        }
        // byte by byte, finish() in between does not change the state
        BitisCrc32 c;
        for (std::size_t i = 0; i < n; ++i) {
            c.update(d + i, 1);
            if (i == n / 2) { EXPECT_EQ(c.finish(), bitis_crc32(d, i + 1)); }
        }
        EXPECT_EQ(c.finish(), v.crc) << "byte by byte";
        // reset
        c.reset();
        EXPECT_EQ(c.finish(), 0u);
        c.update(static_cast<const void *>(d), n);
        EXPECT_EQ(c.finish(), v.crc) << "after reset";
    }
    EXPECT_TRUE(have_check);
}
