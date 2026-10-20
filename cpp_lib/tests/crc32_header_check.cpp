// Compiled with -Wall -Wextra -Wpedantic -Werror -fno-exceptions -fno-rtti (ESP32 firmware
// settings, see CMakeLists.txt), once with the default nibble table and once with
// BITIS_CRC32_TABLE256: bitis_crc32.h must compile warning free and give the check value.
#include "bitis_crc32.h"
#include "bitis_crc32.h"

constexpr uint8_t kCheck[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
static_assert(bitis_crc32(kCheck, sizeof(kCheck)) == 0xCBF43926u, "check value");

constexpr uint32_t incremental() {
    BitisCrc32 c;
    c.update(kCheck, 4);
    c.update(kCheck + 4, 5);
    return c.finish();
}
static_assert(incremental() == 0xCBF43926u, "incremental");

int main() {
    BitisCrc32 c;
    c.update("1234", 4);
    c.update("56789", 5);
    const bool ok = c.finish() == 0xCBF43926u && bitis_crc32("123456789", 9) == 0xCBF43926u;
    c.reset();
    return ok && c.finish() == 0u ? 0 : 1;
}
