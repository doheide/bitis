// bitis CRC-32 helper (header only, independent of bitis_lib.h; not part of the wire format)
//
// CRC-32/ISO-HDLC: reflected polynomial 0xEDB88320, init 0xFFFFFFFF, xorout 0xFFFFFFFF,
// reflected input and output -- the CRC of zlib / Python zlib.crc32, Ethernet, PNG, gzip.
// Check value: bitis_crc32("123456789") == 0xCBF43926, bitis_crc32("") == 0.
//
//   uint32_t c = bitis_crc32(buf, len);                     // one-shot
//   BitisCrc32 crc; crc.update(a, na); crc.update(b, nb);   // incremental, any split
//   uint32_t c2 = crc.finish();  crc.reset();
//
// ESP32: the ROM function computes the same CRC, use it on the device:
//   uint32_t c = esp_rom_crc32_le(0, buf, len);             // == bitis_crc32(buf, len)
//   incremental: uint32_t c = 0; c = esp_rom_crc32_le(c, a, na); c = esp_rom_crc32_le(c, b, nb);
// (esp_rom_crc32_le inverts the CRC on entry and exit, so the "previous result" is passed in;
// do NOT pass ~0 / 0xFFFFFFFF as start value and do NOT invert the result.)
// This header is meant for host tools, tests and as a portable fallback.
//
// Table size: by default a 16-entry nibble table (64 bytes of read-only data, two lookups
// per byte), which is small enough for any microcontroller. Define BITIS_CRC32_TABLE256
// before including this header to use a 256-entry table instead (1 KiB, one lookup per
// byte, about twice as fast). Both tables are computed at compile time (constexpr).
//
// Both variants live in different inline namespaces (bitis_crc32_nibble16 /
// bitis_crc32_table256), so translation units of one program may choose differently
// without violating the one definition rule.
//
// Requirements: C++17; no heap, exceptions or RTTI; warning free with -Wall -Wextra -Wpedantic.
#ifndef BITIS_CRC32_H
#define BITIS_CRC32_H

#include <cstddef>
#include <cstdint>

#ifdef BITIS_CRC32_TABLE256
inline namespace bitis_crc32_table256 {
#else
inline namespace bitis_crc32_nibble16 {
#endif

namespace bitis_crc32_detail {
    constexpr uint32_t kPoly = 0xEDB88320u;

    template<std::size_t N>
    struct Table { uint32_t v[N]; };

    // Table of N = 2^bits entries: CRC register after shifting in the `bits` low bits of i.
    template<std::size_t N, unsigned Bits>
    constexpr Table<N> make_table() {
        Table<N> t{};
        for (std::size_t i = 0; i < N; ++i) {
            uint32_t c = static_cast<uint32_t>(i);
            for (unsigned k = 0; k < Bits; ++k) {
                c = (c & 1u) ? ((c >> 1) ^ kPoly) : (c >> 1);
            }
            t.v[i] = c;
        }
        return t;
    }

#ifdef BITIS_CRC32_TABLE256
    inline constexpr Table<256> kTable = make_table<256, 8>();
#else
    inline constexpr Table<16> kTable = make_table<16, 4>();
#endif

    // Feeds len bytes into the CRC register c (no init / xorout).
    constexpr uint32_t update(uint32_t c, const uint8_t *data, std::size_t len) {
        for (std::size_t i = 0; i < len; ++i) {
#ifdef BITIS_CRC32_TABLE256
            c = kTable.v[(c ^ data[i]) & 0xFFu] ^ (c >> 8);
#else
            c ^= data[i];
            c = kTable.v[c & 0x0Fu] ^ (c >> 4);
            c = kTable.v[c & 0x0Fu] ^ (c >> 4);
#endif
        }
        return c;
    }
}

// Incremental CRC-32/ISO-HDLC: call update() any number of times (the input may be split
// arbitrarily), finish() returns the CRC of everything passed since construction / reset().
class BitisCrc32 {
public:
    constexpr BitisCrc32() = default;

    constexpr void update(const uint8_t *data, std::size_t len) {
        state_ = bitis_crc32_detail::update(state_, data, len);
    }
    // for char buffers, structs, ...
    void update(const void *data, std::size_t len) {
        update(static_cast<const uint8_t *>(data), len);
    }
    // Does not change the state: more data can be appended afterwards.
    constexpr uint32_t finish() const { return state_ ^ 0xFFFFFFFFu; }
    constexpr void reset() { state_ = 0xFFFFFFFFu; }

private:
    uint32_t state_ = 0xFFFFFFFFu;
};

// CRC-32/ISO-HDLC of len bytes at data (same value as zlib crc32(0, data, len)).
constexpr uint32_t bitis_crc32(const uint8_t *data, std::size_t len) {
    return bitis_crc32_detail::update(0xFFFFFFFFu, data, len) ^ 0xFFFFFFFFu;
}
inline uint32_t bitis_crc32(const void *data, std::size_t len) {
    return bitis_crc32(static_cast<const uint8_t *>(data), len);
}

} // inline namespace

#endif // BITIS_CRC32_H
