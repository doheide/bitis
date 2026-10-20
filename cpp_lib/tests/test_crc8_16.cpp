// Tests of the CRC-8/AUTOSAR, CRC-16/IBM-3740 and transmission CRC helpers bitis_crc8_16.h
// against the shared vectors golden/vectors/crc8_16.txt (path given by BITIS_GOLDEN_CRC8_16,
// see golden/README.md): one-shot, incremental with several splits, byte by byte, reset;
// frames with trailer / append / verify / strip; the documented ESP32 ROM call.
#include <gtest/gtest.h>

#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "bitis_crc8_16.h"

#ifndef BITIS_GOLDEN_CRC8_16
#error "BITIS_GOLDEN_CRC8_16 (path of golden/vectors/crc8_16.txt) has to be defined"
#endif

namespace {

struct Crc816Vector {
    std::string name;
    std::vector<uint8_t> data;
    uint8_t crc8 = 0;
    uint16_t crc16 = 0;
};

struct Crc816Frame {
    std::string name;
    std::vector<uint8_t> header, payload, trailer;
};

std::vector<uint8_t> unhex(const std::string &s) {
    std::vector<uint8_t> out;
    if (s == "-") { return out; }
    EXPECT_EQ(s.size() % 2, 0u) << s;
    for (std::size_t i = 0; i + 1 < s.size(); i += 2) {
        out.push_back(static_cast<uint8_t>(std::stoul(s.substr(i, 2), nullptr, 16)));
    }
    return out;
}

void load(std::vector<Crc816Vector> &vectors, std::vector<Crc816Frame> &frames) {
    std::ifstream in(BITIS_GOLDEN_CRC8_16);
    EXPECT_TRUE(in.good()) << "cannot open " << BITIS_GOLDEN_CRC8_16;
    std::string line;
    while (std::getline(in, line)) {
        if (line.empty() || line[0] == '#') { continue; }
        std::istringstream ls(line);
        std::string kind, name, a, b, c, rest;
        ls >> kind >> name >> a >> b >> c;
        EXPECT_FALSE(ls >> rest) << line;
        if (kind == "frame") {
            frames.push_back({name, unhex(a), unhex(b), unhex(c)});
            continue;
        }
        Crc816Vector v;
        v.name = name;
        if (kind == "hex") {
            v.data = unhex(a);
        } else if (kind == "ascii") {
            v.data.assign(a.begin(), a.end());
        } else {
            ADD_FAILURE() << "unknown record kind: " << line;
            continue;
        }
        EXPECT_EQ(b.size(), 2u) << line;
        EXPECT_EQ(c.size(), 4u) << line;
        v.crc8 = static_cast<uint8_t>(std::stoul(b, nullptr, 16));
        v.crc16 = static_cast<uint16_t>(std::stoul(c, nullptr, 16));
        vectors.push_back(v);
    }
}

// Model of the ESP32 ROM function esp_rom_crc16_be (esp-idf components/esp_rom/linux/
// esp_rom_crc.c and patches/esp_rom_crc.c: crc = ~crc; table loop over poly 0x1021,
// MSB first; return ~crc), bit-wise instead of with its table.
uint16_t esp_rom_crc16_be_model(uint16_t crc, const uint8_t *buf, uint32_t len) {
    crc = static_cast<uint16_t>(~crc);
    for (uint32_t i = 0; i < len; ++i) {
        crc = static_cast<uint16_t>(crc ^ (buf[i] << 8));
        for (int k = 0; k < 8; ++k) {
            crc = static_cast<uint16_t>((crc & 0x8000u) ? ((crc << 1) ^ 0x1021u) : (crc << 1));
        }
    }
    return static_cast<uint16_t>(~crc);
}

// constexpr evaluation
constexpr uint8_t kCheck[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
static_assert(bitis_crc8_autosar(kCheck, sizeof(kCheck)) == 0xDFu, "crc8 check value");
static_assert(bitis_crc8_autosar(kCheck, 0) == 0x00u, "crc8 empty input");
static_assert(bitis_crc16_ibm3740(kCheck, sizeof(kCheck)) == 0x29B1u, "crc16 check value");
static_assert(bitis_crc16_ibm3740(kCheck, 0) == 0xFFFFu, "crc16 empty input");
static_assert(bitis_transmission_crc_len(14) == 1 && bitis_transmission_crc_len(15) == 2, "boundary");

} // namespace

TEST(Crc8_16, CheckValues) {
    EXPECT_EQ(bitis_crc8_autosar("123456789", 9), 0xDFu);
    EXPECT_EQ(bitis_crc16_ibm3740("123456789", 9), 0x29B1u);
    EXPECT_EQ(BitisCrc8Autosar().finish(), 0x00u);
    EXPECT_EQ(BitisCrc16Ibm3740().finish(), 0xFFFFu);
    const std::string s = "123456789";
    EXPECT_EQ(bitis_crc8_autosar(s.data(), s.size()), 0xDFu);
    EXPECT_EQ(bitis_crc16_ibm3740(s.data(), s.size()), 0x29B1u);
}

TEST(Crc8_16, Vectors) {
    std::vector<Crc816Vector> vectors;
    std::vector<Crc816Frame> frames;
    load(vectors, frames);
    ASSERT_GE(vectors.size(), 16u);
    bool have_check = false;
    for (const auto &v : vectors) {
        SCOPED_TRACE(v.name);
        const uint8_t *d = v.data.data();
        const std::size_t n = v.data.size();
        if (v.name == "check") {
            have_check = true;
            EXPECT_EQ(v.crc8, 0xDFu);
            EXPECT_EQ(v.crc16, 0x29B1u);
        }
        EXPECT_EQ(bitis_crc8_autosar(d, n), v.crc8);
        EXPECT_EQ(bitis_crc16_ibm3740(d, n), v.crc16);

        // splits at the start, the end, the middle and every position of short inputs
        std::vector<std::size_t> cuts = {0, 1, n / 2, n > 0 ? n - 1 : 0, n};
        if (n <= 64) { for (std::size_t i = 0; i <= n; ++i) { cuts.push_back(i); } }
        for (std::size_t i : cuts) {
            if (i > n) { continue; }
            BitisCrc8Autosar c8;
            BitisCrc16Ibm3740 c16;
            c8.update(d, i);
            c8.update(d + i, n - i);
            c16.update(d, i);
            c16.update(d + i, n - i);
            EXPECT_EQ(c8.finish(), v.crc8) << "split at " << i;
            EXPECT_EQ(c16.finish(), v.crc16) << "split at " << i;
        }
        // three pieces, with empty updates in between
        {
            const std::size_t a = n / 3, b = 2 * n / 3;
            BitisCrc8Autosar c8;
            BitisCrc16Ibm3740 c16;
            c8.update(d, a);
            c8.update(d + a, 0);
            c8.update(d + a, b - a);
            c8.update(d + b, n - b);
            c8.update(d + n, 0);
            c16.update(d, a);
            c16.update(d + a, 0);
            c16.update(d + a, b - a);
            c16.update(d + b, n - b);
            c16.update(d + n, 0);
            EXPECT_EQ(c8.finish(), v.crc8) << "three pieces";
            EXPECT_EQ(c16.finish(), v.crc16) << "three pieces";
        }
        // byte by byte, finish() in between does not change the state
        BitisCrc8Autosar c8;
        BitisCrc16Ibm3740 c16;
        for (std::size_t i = 0; i < n; ++i) {
            c8.update(d + i, 1);
            c16.update(d + i, 1);
            if (i == n / 2) {
                EXPECT_EQ(c8.finish(), bitis_crc8_autosar(d, i + 1));
                EXPECT_EQ(c16.finish(), bitis_crc16_ibm3740(d, i + 1));
            }
        }
        EXPECT_EQ(c8.finish(), v.crc8) << "byte by byte";
        EXPECT_EQ(c16.finish(), v.crc16) << "byte by byte";
        // reset
        c8.reset();
        c16.reset();
        EXPECT_EQ(c8.finish(), 0x00u);
        EXPECT_EQ(c16.finish(), 0xFFFFu);
        c8.update(static_cast<const void *>(d), n);
        c16.update(static_cast<const void *>(d), n);
        EXPECT_EQ(c8.finish(), v.crc8) << "after reset";
        EXPECT_EQ(c16.finish(), v.crc16) << "after reset";
        // property of the CRC function itself (not used by the transmission rule, whose trailer
        // is little-endian): the CRC-16 over data + CRC big-endian is 0
        std::vector<uint8_t> with = v.data;
        with.push_back(static_cast<uint8_t>(v.crc16 >> 8));
        with.push_back(static_cast<uint8_t>(v.crc16 & 0xFFu));
        EXPECT_EQ(bitis_crc16_ibm3740(with.data(), with.size()), 0u) << "residue";
        // ESP32 ROM equivalence (README): (uint16_t)~esp_rom_crc16_be(0, buf, len), incremental
        // by passing the previous return value
        EXPECT_EQ(static_cast<uint16_t>(~esp_rom_crc16_be_model(0, d, static_cast<uint32_t>(n))), v.crc16);
        uint16_t r = 0;
        r = esp_rom_crc16_be_model(r, d, static_cast<uint32_t>(n / 2));
        r = esp_rom_crc16_be_model(r, d + n / 2, static_cast<uint32_t>(n - n / 2));
        EXPECT_EQ(static_cast<uint16_t>(~r), v.crc16) << "esp rom incremental";
    }
    EXPECT_TRUE(have_check);
}

TEST(Crc8_16, TransmissionFrames) {
    std::vector<Crc816Vector> vectors;
    std::vector<Crc816Frame> frames;
    load(vectors, frames);
    ASSERT_GE(frames.size(), 11u);
    bool have_n14 = false, have_n15 = false;
    for (const auto &f : frames) {
        SCOPED_TRACE(f.name);
        std::vector<uint8_t> frame = f.header;
        frame.insert(frame.end(), f.payload.begin(), f.payload.end());
        const std::size_t n = frame.size();
        have_n14 = have_n14 || n == 14;
        have_n15 = have_n15 || n == 15;
        for (std::size_t i = 0; i < f.payload.size(); ++i) {
            ASSERT_EQ(f.payload[i], static_cast<uint8_t>(i * 37 + 1));
        }
        ASSERT_EQ(bitis_transmission_crc_len(n), f.trailer.size());
        const uint16_t crc = bitis_transmission_crc(frame.data(), n);
        if (n <= 14) {
            EXPECT_EQ(crc, bitis_crc8_autosar(frame.data(), n));
            EXPECT_EQ(crc, f.trailer[0]);
        } else {
            EXPECT_EQ(crc, bitis_crc16_ibm3740(frame.data(), n));
            EXPECT_EQ(crc, f.trailer[0] | (f.trailer[1] << 8)) << "little-endian";
        }
        uint8_t trailer[2] = {0, 0};
        ASSERT_EQ(bitis_transmission_crc_trailer(frame.data(), n, trailer), f.trailer.size());
        EXPECT_TRUE(std::equal(f.trailer.begin(), f.trailer.end(), trailer));

        std::vector<uint8_t> buf = frame;
        buf.resize(n + 2, 0xEE);
        const std::size_t len = bitis_append_transmission_crc(buf.data(), n);
        ASSERT_EQ(len, n + f.trailer.size());
        EXPECT_TRUE(std::equal(f.trailer.begin(), f.trailer.end(), buf.begin() + static_cast<std::ptrdiff_t>(n)));
        if (len == n + 1) { EXPECT_EQ(buf[n + 1], 0xEEu) << "wrote past the trailer"; }
        EXPECT_TRUE(bitis_verify_transmission_crc(buf.data(), len));
        std::size_t m = 12345;
        EXPECT_TRUE(bitis_strip_transmission_crc(buf.data(), len, &m));
        EXPECT_EQ(m, n);
        EXPECT_TRUE(bitis_strip_transmission_crc(static_cast<const void *>(buf.data()), len, nullptr));

        // every single-bit error is detected, *n is not written on failure
        for (std::size_t bit = 0; bit < len * 8; ++bit) {
            buf[bit / 8] = static_cast<uint8_t>(buf[bit / 8] ^ (1u << (bit % 8)));
            m = 12345;
            EXPECT_FALSE(bitis_strip_transmission_crc(buf.data(), len, &m)) << "bit " << bit;
            EXPECT_EQ(m, 12345u);
            buf[bit / 8] = static_cast<uint8_t>(buf[bit / 8] ^ (1u << (bit % 8)));
        }
    }
    EXPECT_TRUE(have_n14 && have_n15);
}

TEST(Crc8_16, TransmissionLengths) {
    EXPECT_EQ(bitis_transmission_crc_len(0), 1u);
    EXPECT_EQ(bitis_transmission_crc_len(14), 1u);
    EXPECT_EQ(bitis_transmission_crc_len(15), 2u);
    EXPECT_EQ(bitis_transmission_crc_len(SIZE_MAX), 2u);

    uint8_t buf[20] = {};
    for (auto &b : buf) { b = 7; }
    // L = 15 (n = 14, CRC-8) and L = 17 (n = 15, CRC-16)
    EXPECT_EQ(bitis_append_transmission_crc(buf, 14), 15u);
    EXPECT_TRUE(bitis_verify_transmission_crc(buf, 15));
    for (auto &b : buf) { b = 7; }
    EXPECT_EQ(bitis_append_transmission_crc(buf, 15), 17u);
    EXPECT_TRUE(bitis_verify_transmission_crc(buf, 17));
    // L = 16 is invalid, also when a CRC-16 over 14 bytes or a CRC-8 over 15 bytes matches
    for (auto &b : buf) { b = 7; }
    const uint16_t c16 = bitis_crc16_ibm3740(buf, 14);
    buf[14] = static_cast<uint8_t>(c16 & 0xFFu);
    buf[15] = static_cast<uint8_t>(c16 >> 8);
    EXPECT_FALSE(bitis_verify_transmission_crc(buf, 16));
    buf[15] = bitis_crc8_autosar(buf, 15);
    std::size_t m = 99;
    EXPECT_FALSE(bitis_strip_transmission_crc(buf, 16, &m));
    EXPECT_EQ(m, 99u);
    // empty input / n = 0
    EXPECT_FALSE(bitis_verify_transmission_crc(buf, 0));
    uint8_t one[2] = {0x55, 0x55};
    EXPECT_EQ(bitis_append_transmission_crc(one, 0), 1u);
    EXPECT_EQ(one[0], 0x00u);
    EXPECT_TRUE(bitis_strip_transmission_crc(one, 1, &m));
    EXPECT_EQ(m, 0u);
    one[0] = 0x01;
    EXPECT_FALSE(bitis_verify_transmission_crc(one, 1));
}
