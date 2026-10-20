// bitis CRC-8/AUTOSAR, CRC-16/IBM-3740 and transmission CRC helpers (header only,
// independent of bitis_lib.h and bitis_crc32.h; not part of the wire format)
//
// CRC-8/AUTOSAR:    poly 0x2F,   init 0xFF,   xorout 0xFF,   no reflection;
//                   bitis_crc8_autosar("123456789") == 0xDF, of "" == 0x00.
// CRC-16/IBM-3740:  poly 0x1021, init 0xFFFF, xorout 0x0000, no reflection (CCITT-FALSE);
//                   bitis_crc16_ibm3740("123456789") == 0x29B1, of "" == 0xFFFF.
//
//   uint8_t  c8  = bitis_crc8_autosar(buf, len);               // one-shot
//   uint16_t c16 = bitis_crc16_ibm3740(buf, len);
//   BitisCrc16Ibm3740 crc; crc.update(a, na); crc.update(b, nb);  // incremental, any split
//   uint16_t c = crc.finish();  crc.reset();                    // (BitisCrc8Autosar alike)
//
// Transmission CRC (carachangprotocol FRAMING.md section 2): a frame of n bytes (header +
// payload as on the wire, the CRC not counted) gets a 1-byte CRC-8/AUTOSAR trailer for
// n <= 14 and a 2-byte CRC-16/IBM-3740 trailer, little-endian (low byte first: 0x29B1 is
// sent as B1 29), for n >= 15. A received frame of L bytes (trailer included) uses CRC-8 for
// L <= 15 and CRC-16 for L >= 17; L = 16 and L = 0 are invalid. n = 0 is allowed (trailer
// 0x00); real frames always have a header. The receiver recomputes the CRC over header +
// payload and compares it with the trailer (the "CRC over frame + trailer = 0" residue check
// does not work with the little-endian trailer).
//
//   uint8_t buf[2 + 2047 + 2];                               // room for the trailer
//   size_t n = ...;                                          // header + payload in buf
//   size_t len = bitis_append_transmission_crc(buf, n);      // writes buf[n], buf[n+1]
//
//   size_t n;                                                // receiver, whole frame:
//   if (!bitis_strip_transmission_crc(rx, rx_len, &n)) { /* close connection */ }
//   // rx[0 .. n) is header + payload
//
// A stream receiver knows n after parsing the header and reads
// bitis_transmission_crc_len(n) trailer bytes. Header and payload in separate buffers:
// pick BitisCrc8Autosar or BitisCrc16Ibm3740 by bitis_transmission_crc_len(n) and update()
// it with both parts.
//
// ESP32: the ROM computes CRC-16/IBM-3740 (esp_rom_crc.h, also esp_crc16_be in esp_crc.h):
//   uint16_t c = (uint16_t)~esp_rom_crc16_be(0, buf, len);      // == bitis_crc16_ibm3740
//   incremental: uint16_t r = 0; r = esp_rom_crc16_be(r, a, na); r = esp_rom_crc16_be(r, b, nb);
//                uint16_t c = (uint16_t)~r;
// (esp_rom_crc16_be inverts the register on entry and exit: start value 0 means register
// 0xFFFF; the final ~ undoes the exit inversion because xorout is 0. Keep the (uint16_t)
// cast: ~ on a uint16_t gives an int with the upper bits set.) There is no ROM function
// for CRC-8/AUTOSAR (the ROM CRC-8 uses polynomial 0x07): use bitis_crc8_autosar.
//
// Tables: 16-entry nibble tables, 16 bytes (CRC-8) and 32 bytes (CRC-16) of read-only
// data, two lookups per byte, computed at compile time (constexpr). Frames are at most
// about 2 KiB, so a 256-entry table would not pay off on a microcontroller.
//
// Requirements: C++17; no heap, exceptions or RTTI; warning free with -Wall -Wextra -Wpedantic.
#ifndef BITIS_CRC8_16_H
#define BITIS_CRC8_16_H

#include <cstddef>
#include <cstdint>

// versioned like bitis_crc32.h, so that another table variant could be added later without
// violating the one definition rule
inline namespace bitis_crc8_16_nibble16 {

namespace bitis_crc8_16_detail {
    constexpr uint8_t kPoly8 = 0x2Fu;
    constexpr uint16_t kPoly16 = 0x1021u;
    // largest n (bytes without the CRC) protected by CRC-8
    constexpr std::size_t kCrc8MaxN = 14;

    template<typename T>
    struct Table16 { T v[16]; };

    // v[i] = CRC-8 register after shifting in the 4 bits of i (register i << 4)
    constexpr Table16<uint8_t> make_table8() {
        Table16<uint8_t> t{};
        for (unsigned i = 0; i < 16; ++i) {
            unsigned c = i << 4;
            for (unsigned k = 0; k < 4; ++k) {
                c = (c & 0x80u) ? ((c << 1) ^ kPoly8) : (c << 1);
            }
            t.v[i] = static_cast<uint8_t>(c & 0xFFu);
        }
        return t;
    }

    // v[i] = CRC-16 register after shifting in the 4 bits of i (register i << 12)
    constexpr Table16<uint16_t> make_table16() {
        Table16<uint16_t> t{};
        for (unsigned i = 0; i < 16; ++i) {
            uint32_t c = static_cast<uint32_t>(i) << 12;
            for (unsigned k = 0; k < 4; ++k) {
                c = (c & 0x8000u) ? ((c << 1) ^ kPoly16) : (c << 1);
            }
            t.v[i] = static_cast<uint16_t>(c & 0xFFFFu);
        }
        return t;
    }

    inline constexpr Table16<uint8_t> kTable8 = make_table8();
    inline constexpr Table16<uint16_t> kTable16 = make_table16();

    // Feed len bytes into the CRC register c (no init / xorout).
    constexpr uint8_t update8(uint8_t c, const uint8_t *data, std::size_t len) {
        unsigned r = c;
        for (std::size_t i = 0; i < len; ++i) {
            r ^= data[i];
            r = ((r << 4) & 0xF0u) ^ kTable8.v[r >> 4];
            r = ((r << 4) & 0xF0u) ^ kTable8.v[r >> 4];
        }
        return static_cast<uint8_t>(r);
    }

    constexpr uint16_t update16(uint16_t c, const uint8_t *data, std::size_t len) {
        uint32_t r = c;
        for (std::size_t i = 0; i < len; ++i) {
            r ^= static_cast<uint32_t>(data[i]) << 8;
            r = ((r << 4) & 0xFFF0u) ^ kTable16.v[r >> 12];
            r = ((r << 4) & 0xFFF0u) ^ kTable16.v[r >> 12];
        }
        return static_cast<uint16_t>(r);
    }
}

// ---------------------------------------------------------------------------------------------
// CRC-8/AUTOSAR

// Incremental CRC-8/AUTOSAR: call update() any number of times (the input may be split
// arbitrarily), finish() returns the CRC of everything passed since construction / reset().
class BitisCrc8Autosar {
public:
    constexpr BitisCrc8Autosar() = default;

    constexpr void update(const uint8_t *data, std::size_t len) {
        state_ = bitis_crc8_16_detail::update8(state_, data, len);
    }
    // for char buffers, structs, ...
    void update(const void *data, std::size_t len) {
        update(static_cast<const uint8_t *>(data), len);
    }
    // Does not change the state: more data can be appended afterwards.
    constexpr uint8_t finish() const { return static_cast<uint8_t>(state_ ^ 0xFFu); }
    constexpr void reset() { state_ = 0xFFu; }

private:
    uint8_t state_ = 0xFFu;
};

// CRC-8/AUTOSAR of len bytes at data.
constexpr uint8_t bitis_crc8_autosar(const uint8_t *data, std::size_t len) {
    return static_cast<uint8_t>(bitis_crc8_16_detail::update8(0xFFu, data, len) ^ 0xFFu);
}
inline uint8_t bitis_crc8_autosar(const void *data, std::size_t len) {
    return bitis_crc8_autosar(static_cast<const uint8_t *>(data), len);
}

// ---------------------------------------------------------------------------------------------
// CRC-16/IBM-3740

// Incremental CRC-16/IBM-3740 (CCITT-FALSE), see BitisCrc8Autosar.
class BitisCrc16Ibm3740 {
public:
    constexpr BitisCrc16Ibm3740() = default;

    constexpr void update(const uint8_t *data, std::size_t len) {
        state_ = bitis_crc8_16_detail::update16(state_, data, len);
    }
    // for char buffers, structs, ...
    void update(const void *data, std::size_t len) {
        update(static_cast<const uint8_t *>(data), len);
    }
    // Does not change the state: more data can be appended afterwards.
    constexpr uint16_t finish() const { return state_; }
    constexpr void reset() { state_ = 0xFFFFu; }

private:
    uint16_t state_ = 0xFFFFu;
};

// CRC-16/IBM-3740 (CCITT-FALSE) of len bytes at data.
constexpr uint16_t bitis_crc16_ibm3740(const uint8_t *data, std::size_t len) {
    return bitis_crc8_16_detail::update16(0xFFFFu, data, len);
}
inline uint16_t bitis_crc16_ibm3740(const void *data, std::size_t len) {
    return bitis_crc16_ibm3740(static_cast<const uint8_t *>(data), len);
}

// ---------------------------------------------------------------------------------------------
// transmission CRC rule (carachangprotocol FRAMING.md section 2)

// Trailer length for a frame of n bytes (header + payload, without the CRC): 1 for n <= 14
// (CRC-8/AUTOSAR), 2 for n >= 15 (CRC-16/IBM-3740, little-endian).
constexpr std::size_t bitis_transmission_crc_len(std::size_t n) {
    return n <= bitis_crc8_16_detail::kCrc8MaxN ? 1u : 2u;
}

// Transmission CRC of the n-byte frame: the CRC-8 (zero-extended) for n <= 14, else the CRC-16.
constexpr uint16_t bitis_transmission_crc(const uint8_t *frame, std::size_t n) {
    return n <= bitis_crc8_16_detail::kCrc8MaxN ? bitis_crc8_autosar(frame, n)
                                                : bitis_crc16_ibm3740(frame, n);
}
inline uint16_t bitis_transmission_crc(const void *frame, std::size_t n) {
    return bitis_transmission_crc(static_cast<const uint8_t *>(frame), n);
}

// Writes the trailer of the n-byte frame to out_trailer (room for 2 bytes; the CRC-16 low
// byte first) and returns its length (1 or 2). frame and out_trailer may be the same buffer offset by n.
constexpr std::size_t bitis_transmission_crc_trailer(const uint8_t *frame, std::size_t n,
                                                     uint8_t *out_trailer) {
    const uint16_t crc = bitis_transmission_crc(frame, n);
    if (n <= bitis_crc8_16_detail::kCrc8MaxN) {
        out_trailer[0] = static_cast<uint8_t>(crc);
        return 1;
    }
    out_trailer[0] = static_cast<uint8_t>(crc & 0xFFu);
    out_trailer[1] = static_cast<uint8_t>(crc >> 8);
    return 2;
}
inline std::size_t bitis_transmission_crc_trailer(const void *frame, std::size_t n,
                                                  void *out_trailer) {
    return bitis_transmission_crc_trailer(static_cast<const uint8_t *>(frame), n,
                                          static_cast<uint8_t *>(out_trailer));
}

// buf holds the n-byte frame and has room for n + 2 bytes: appends the trailer at buf + n and
// returns the length of the frame with trailer (n + 1 or n + 2).
constexpr std::size_t bitis_append_transmission_crc(uint8_t *buf, std::size_t n) {
    return n + bitis_transmission_crc_trailer(buf, n, buf + n);
}
inline std::size_t bitis_append_transmission_crc(void *buf, std::size_t n) {
    return bitis_append_transmission_crc(static_cast<uint8_t *>(buf), n);
}

// Checks a received frame of len bytes (trailer included). Returns false for len 0 or 16 and
// for a CRC mismatch; on success stores the length without the trailer in *n (if n is not
// null).
constexpr bool bitis_strip_transmission_crc(const uint8_t *data, std::size_t len, std::size_t *n) {
    constexpr std::size_t kMax = bitis_crc8_16_detail::kCrc8MaxN;
    if (len == 0 || len == kMax + 2) { return false; }
    std::size_t m = 0;
    if (len <= kMax + 1) {
        m = len - 1;
        if (bitis_crc8_autosar(data, m) != data[m]) { return false; }
    } else {
        m = len - 2;
        const uint16_t crc = bitis_crc16_ibm3740(data, m);
        if (static_cast<uint8_t>(crc & 0xFFu) != data[m] || static_cast<uint8_t>(crc >> 8) != data[m + 1]) {
            return false;
        }
    }
    if (n != nullptr) { *n = m; }
    return true;
}
inline bool bitis_strip_transmission_crc(const void *data, std::size_t len, std::size_t *n) {
    return bitis_strip_transmission_crc(static_cast<const uint8_t *>(data), len, n);
}

// true if the received frame (trailer included) has a valid length and a matching CRC.
constexpr bool bitis_verify_transmission_crc(const uint8_t *data, std::size_t len) {
    return bitis_strip_transmission_crc(data, len, nullptr);
}
inline bool bitis_verify_transmission_crc(const void *data, std::size_t len) {
    return bitis_verify_transmission_crc(static_cast<const uint8_t *>(data), len);
}

} // inline namespace

#endif // BITIS_CRC8_16_H
