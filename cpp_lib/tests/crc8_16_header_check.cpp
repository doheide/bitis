// Compiled with -Wall -Wextra -Wpedantic -Werror -fno-exceptions -fno-rtti (ESP32 firmware
// settings, see CMakeLists.txt): bitis_crc8_16.h must compile warning free (also together
// with bitis_crc32.h) and give the check values, also in constant expressions.
#include "bitis_crc8_16.h"
#include "bitis_crc8_16.h"
#include "bitis_crc32.h"

constexpr uint8_t kCheck[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
static_assert(bitis_crc8_autosar(kCheck, sizeof(kCheck)) == 0xDFu, "crc8 check value");
static_assert(bitis_crc16_ibm3740(kCheck, sizeof(kCheck)) == 0x29B1u, "crc16 check value");
static_assert(bitis_crc32(kCheck, sizeof(kCheck)) == 0xCBF43926u, "crc32 check value");

constexpr bool incremental() {
    BitisCrc8Autosar c8;
    BitisCrc16Ibm3740 c16;
    c8.update(kCheck, 4);
    c8.update(kCheck + 4, 5);
    c16.update(kCheck, 4);
    c16.update(kCheck + 4, 5);
    return c8.finish() == 0xDFu && c16.finish() == 0x29B1u;
}
static_assert(incremental(), "incremental");

// FRAMING.md section 5: header 02, payload 01 -> trailer 95; header 7E + 31 bytes -> F9 AF
constexpr bool frames() {
    uint8_t a[4] = {0x02, 0x01, 0, 0};
    if (bitis_append_transmission_crc(a, 2) != 3 || a[2] != 0x95) { return false; }
    uint8_t b[34] = {0x7E};
    for (unsigned i = 0; i < 31; ++i) { b[1 + i] = static_cast<uint8_t>(i * 37 + 1); }
    if (bitis_append_transmission_crc(b, 32) != 34 || b[32] != 0xF9 || b[33] != 0xAF) { return false; }
    std::size_t n = 0;
    return bitis_strip_transmission_crc(a, 3, &n) && n == 2 && bitis_verify_transmission_crc(b, 34) &&
           !bitis_verify_transmission_crc(b, 16) && !bitis_verify_transmission_crc(b, 0);
}
static_assert(frames(), "transmission frames");

int main() {
    BitisCrc16Ibm3740 c;
    c.update("1234", 4);
    c.update("56789", 5);
    BitisCrc8Autosar c8;
    c8.update("123456789", 9);
    char frame[4] = {0x02, 0x01, 0, 0};
    const std::size_t len = bitis_append_transmission_crc(frame, 2);
    std::size_t n = 0;
    const bool ok = c.finish() == 0x29B1u && c8.finish() == 0xDFu && bitis_crc16_ibm3740("123456789", 9) == 0x29B1u &&
                    len == 3 && bitis_strip_transmission_crc(frame, len, &n) && n == 2;
    c.reset();
    c8.reset();
    return ok && c.finish() == 0xFFFFu && c8.finish() == 0u ? 0 : 1;
}
