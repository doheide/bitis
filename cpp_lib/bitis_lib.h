// bitis C++ runtime (header only)
//
// Wire format: docs/WIRE_FORMAT.md (normative). Requirements: C++17, works without
// exceptions and RTTI (ESP32 firmware). Float/double must be IEEE 754; the bit
// stream is produced arithmetically, so the code does not depend on the byte order
// of the host.
//
// Error handling (bitis 0.11):
//  - BitisDeserializer / BitisSerializer carry a sticky error (first error wins).
//    Reads past the end of the input return 0 and set BitisError::UnexpectedEnd.
//  - try_deserialize<T>(), try_deserialize_exact<T>() and try_serialize<T>() report a
//    BitisError. The legacy functions serialize(), deserialize() and
//    deserialize_with_size() keep their signatures: serialize() silently truncates
//    out-of-range values, deserialize() returns a default constructed T on any error
//    (deserialize_with_size() additionally reports 0 bits).
//  - try_serialize(msg, buf, cap) writes into a caller-provided buffer and fails with
//    BitisError::BufferTooSmall if the message does not fit (C++ only).
//    bitis_max_bytes<T>() gives a buffer size that fits every valid value of T
//    (BITIS_UNBOUNDED for types containing unbounded strings, binaries or lists).
//
// Footprint (ESP32 firmware):
//  - BITIS_NO_PRINT: if defined before including this header (or a generated header),
//    every print() is an empty inline function and no names (attributes, enum values,
//    oneof elements, types) end up in the binary; printf is not referenced.
//  - The per-type templates only forward their constants to shared non-template
//    functions (namespace bitis_core), so the code exists once per program.
//  - Fixed point values store the raw wire value (smallest unsigned type holding N
//    bits); value(), state() and set() convert on access.
#ifndef BITIS_LIB_H
#define BITIS_LIB_H

#include <cfloat>
#include <cinttypes>
#include <climits>
#include <cstddef>
#include <type_traits>
#include <limits>
#include <cstdint>
#include <vector>
#include <array>
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <new>
#include <tuple>
#include <utility>
#include <inttypes.h>

static_assert(CHAR_BIT == 8, "bitis requires 8 bit bytes");
static_assert(std::numeric_limits<float>::is_iec559 && sizeof(float) == 4,
    "bitis requires IEEE 754 binary32 float");
static_assert(std::numeric_limits<double>::is_iec559 && sizeof(double) == 8,
    "bitis requires IEEE 754 binary64 double");

// keeps the shared functions out of line (can be defined empty before including)
#ifndef BITIS_NOINLINE
#if defined(__GNUC__) || defined(__clang__)
#define BITIS_NOINLINE __attribute__((noinline))
#else
#define BITIS_NOINLINE
#endif
#endif

// name of an enum value / oneof element (only used by print())
#ifndef BITIS_NO_PRINT
#define BITIS_NAME(n) static constexpr const char *name = n;
#else
#define BITIS_NAME(n)
#endif

#ifndef BITIS_NO_PRINT
inline void print_indent(const int16_t indent) {
    for(int i=0; i < indent; i++) { printf(" "); }
}
#else
inline void print_indent(const int16_t) { }
#endif

template<typename T>
constexpr uint64_t calc_bitmask(T bits) {
    static_assert(std::is_integral<T>::value, "calc_bitmask(): T must be an integral type");
    if (bits <= 0) return 0;
    if (bits >= 64) return ~static_cast<uint64_t>(0);
    return (static_cast<uint64_t>(1) << bits) - 1;
}
static const uint8_t bit_masks[] = {0, 1, 3, 7, 15, 31, 63, 127, 255};
static const uint8_t max_bitmask = 8;

// ***************************************************************
enum class BitisError : uint8_t {
    Ok = 0,
    // decoding
    UnexpectedEnd,          // input ended before the message was complete
    InvalidValue,           // value outside of its valid range / non-ASCII string
    InvalidDiscriminant,    // unknown enum or oneof index
    TrailingData,           // strict decode only: more bytes than the message needs
    // encoding
    ValueOutOfRange,        // value does not fit its declared width / base type
    BufferTooSmall,         // C++ only: caller-provided output buffer too small
};

inline const char *bitis_error_name(const BitisError e) {
    switch (e) {
        case BitisError::Ok: return "Ok";
        case BitisError::UnexpectedEnd: return "UnexpectedEnd";
        case BitisError::InvalidValue: return "InvalidValue";
        case BitisError::InvalidDiscriminant: return "InvalidDiscriminant";
        case BitisError::TrailingData: return "TrailingData";
        case BitisError::ValueOutOfRange: return "ValueOutOfRange";
        case BitisError::BufferTooSmall: return "BufferTooSmall";
    }
    return "Unknown";
}

// ***************************************************************
// max_bits (docs/WIRE_FORMAT.md section 4): upper bound of the encoded size in bits
// or BITIS_UNBOUNDED. Every bitis type provides static constexpr std::size_t max_bits().
constexpr std::size_t BITIS_UNBOUNDED = std::numeric_limits<std::size_t>::max();
namespace bitis_helper {
    // saturating arithmetic for max_bits (BITIS_UNBOUNDED absorbs)
    constexpr std::size_t max_bits_add(const std::size_t a, const std::size_t b) {
        return (a == BITIS_UNBOUNDED || b == BITIS_UNBOUNDED || a > BITIS_UNBOUNDED - b) ? BITIS_UNBOUNDED : a + b;
    }
    constexpr std::size_t max_bits_mul(const std::size_t a, const std::size_t n) {
        if (a == 0 || n == 0) { return 0; }
        return (a == BITIS_UNBOUNDED || n == BITIS_UNBOUNDED || a > BITIS_UNBOUNDED / n) ? BITIS_UNBOUNDED : a * n;
    }
    // size of the uint_XdY encoding of v (spec 2.4)
    constexpr std::size_t dyn_bits(uint64_t v, const uint8_t max_bits, const uint8_t bit_packs) {
        if (v == 0) { return 1; }
        std::size_t bits = 1;
        int remaining = max_bits;
        while (true) {
            const int c = (bit_packs < remaining) ? bit_packs : remaining;
            bits += static_cast<std::size_t>(c);
            v = (c >= 64) ? 0 : (v >> c);
            remaining -= c;
            if (remaining == 0) { break; }
            bits += 1;
            if (v == 0) { break; }
        }
        return bits;
    }
}

struct BitisSize {
    std::size_t bits;
    std::size_t bytes;
};

namespace bitis_helper {
    template<typename T> struct dependent_false : std::false_type { };

    // raw bit pattern of an arithmetic value (integers: two's complement of the
    // given width, floats: IEEE bit pattern)
    template<typename T>
    inline uint64_t to_u64_bits(const T &v) {
        if constexpr (std::is_same<T, bool>::value) { return v ? 1 : 0; }
        else if constexpr (std::is_integral<T>::value) {
            return static_cast<uint64_t>(static_cast<typename std::make_unsigned<T>::type>(v));
        }
        else if constexpr (std::is_same<T, float>::value) {
            uint32_t u; std::memcpy(&u, &v, sizeof(u)); return u;
        }
        else if constexpr (std::is_same<T, double>::value) {
            uint64_t u; std::memcpy(&u, &v, sizeof(u)); return u;
        }
        else {
            static_assert(dependent_false<T>::value, "to_u64_bits(): unsupported type");
            return 0;
        }
    }
    template<typename T>
    inline T from_u64_bits(const uint64_t u) {
        if constexpr (std::is_same<T, bool>::value) { return u != 0; }
        else if constexpr (std::is_integral<T>::value) {
            return static_cast<T>(static_cast<typename std::make_unsigned<T>::type>(u));
        }
        else if constexpr (std::is_same<T, float>::value) {
            const auto u32 = static_cast<uint32_t>(u); T v; std::memcpy(&v, &u32, sizeof(v)); return v;
        }
        else if constexpr (std::is_same<T, double>::value) {
            T v; std::memcpy(&v, &u, sizeof(v)); return v;
        }
        else {
            static_assert(dependent_false<T>::value, "from_u64_bits(): unsupported type");
            return T();
        }
    }

    // magnitude limits for sign-magnitude integers (spec 2.3)
    template<typename T, uint8_t BITS>
    constexpr uint64_t max_magnitude() {
        return std::min<uint64_t>(calc_bitmask(BITS),
            static_cast<uint64_t>(std::numeric_limits<T>::max()));
    }
    // mask used by the legacy (truncating) serialization; truncated values always decode
    template<typename T, uint8_t BITS>
    constexpr uint64_t truncation_mask() {
        return calc_bitmask(std::min<int>(BITS, std::numeric_limits<T>::digits));
    }
    // |v| as unsigned value (no UB for the minimum of signed types)
    template<typename T>
    constexpr uint64_t magnitude(const T v) {
        if constexpr (std::is_signed<T>::value) {
            return (v < 0) ? static_cast<uint64_t>(0) - static_cast<uint64_t>(static_cast<int64_t>(v))
                           : static_cast<uint64_t>(v);
        }
        else { return static_cast<uint64_t>(v); }
    }
    template<typename T>
    constexpr T from_sign_magnitude(const bool is_negative, const uint64_t mag) {
        // mag has to be <= numeric_limits<T>::max()
        if constexpr (std::is_signed<T>::value) {
            return is_negative ? static_cast<T>(-static_cast<int64_t>(mag)) : static_cast<T>(mag);
        }
        else { (void) is_negative; return static_cast<T>(mag); }
    }
}


// ***************************************************************
// Writes into data_cache (default) or into a caller-provided buffer (see
// try_serialize(msg, buf, cap)); both use the same bit writer.
struct BitisSerializer {
    uint16_t cur_cache_u8 = 0;
    uint8_t sub_byte_counter = 0;
    std::vector<uint8_t> data_cache;    // output if no buffer is given
    std::size_t final_total_bits = 0;
    std::size_t total_bits = 0;
    BitisError error = BitisError::Ok;
    bool use_buf = false;               // output goes to out_buf (out_cap bytes)
    uint8_t *out_buf = nullptr;
    std::size_t out_cap = 0;
    std::size_t out_len = 0;            // bytes produced (including those that did not fit)

    BitisSerializer() = default;
    // more than cap bytes of output set BitisError::BufferTooSmall (the bit count
    // continues, finalize() reports the size the message needs)
    BitisSerializer(uint8_t *buf, const std::size_t cap) : use_buf(true), out_buf(buf), out_cap(cap) { }

    void set_error(const BitisError e) { if (error == BitisError::Ok) { error = e; } }
    bool has_error() const { return error != BitisError::Ok; }

    // write the low num_bits (<= 64) bits of v, LSB first
    BITIS_NOINLINE void write_bits(uint64_t v, uint8_t num_bits) {
        if (num_bits > 64) { num_bits = 64; }
        total_bits += num_bits;
        while (num_bits > 0) {
            const uint8_t space = static_cast<uint8_t>(8 - sub_byte_counter);
            const uint8_t take = (num_bits < space) ? num_bits : space;
            cur_cache_u8 = static_cast<uint16_t>(cur_cache_u8 | ((static_cast<uint32_t>(v) & bit_masks[take]) << sub_byte_counter));
            sub_byte_counter = static_cast<uint8_t>(sub_byte_counter + take);
            num_bits = static_cast<uint8_t>(num_bits - take);
            v >>= take;
            if (sub_byte_counter >= 8) { flush_byte(); }
        }
    }

    template<typename T>
    std::size_t add_data_bits(const T *value, uint8_t bits_left) {
        write_bits(bitis_helper::to_u64_bits(*value), bits_left);
        return bits_left;
    }

    template<typename T, std::size_t BITS>
    std::size_t add_data(const T *value) {
        static_assert(BITS <= 64, "add_data(): at most 64 bits");
        return add_data_bits(value, BITS);
    }

    BitisSize finalize() {
        if (sub_byte_counter > 0) { flush_byte(); }
        final_total_bits = total_bits;
        return BitisSize{final_total_bits, (final_total_bits + 7) / 8};
    }

private:
    void flush_byte() {
        const auto b = static_cast<uint8_t>(cur_cache_u8 & 255);
        sub_byte_counter = 0;
        cur_cache_u8 = 0;
        if (!use_buf) { data_cache.push_back(b); }
        else if (out_len < out_cap) { out_buf[out_len] = b; }
        else { set_error(BitisError::BufferTooSmall); }
        out_len++;
    }
};

namespace bitis_helper {
    struct BitiaDeserializerUint8Helper {
        std::size_t bits;
        uint8_t data;
    };
    template<typename T> struct BitiaDeserializerHelper {
        std::size_t bits;
        T data;
    };

    //  convert a type T to a non-cv-qualified type - remove_const<T>
    template <class T> struct remove_const{ typedef T type; };
    template <class T> struct remove_const<T const>{ typedef T type; };
}

// ***************************************************************
struct BitisDeserializer {
    std::size_t cur_read_pos;           // byte index of the next bit
    uint8_t sub_byte_counter;           // bit index (0..7) within that byte
    std::vector<uint8_t> data_cache;    // owned copy of the input (vector constructor only)
    const uint8_t *data_ptr;
    std::size_t data_size;
    BitisError error;
    bool owns_data;

    // copies the data
    explicit BitisDeserializer(const std::vector<uint8_t> &data)
    : cur_read_pos(0), sub_byte_counter(0), data_cache(data), data_ptr(nullptr), data_size(0),
      error(BitisError::Ok), owns_data(true) {
        data_ptr = data_cache.data(); data_size = data_cache.size();
    }
    // non-owning: data has to outlive the deserializer
    BitisDeserializer(const uint8_t *data, const std::size_t size)
    : cur_read_pos(0), sub_byte_counter(0), data_cache(), data_ptr(data), data_size(data ? size : 0),
      error(BitisError::Ok), owns_data(false) { }

    BitisDeserializer(const BitisDeserializer &other)
    : cur_read_pos(other.cur_read_pos), sub_byte_counter(other.sub_byte_counter), data_cache(other.data_cache),
      data_ptr(other.owns_data ? data_cache.data() : other.data_ptr), data_size(other.data_size),
      error(other.error), owns_data(other.owns_data) { }
    BitisDeserializer(BitisDeserializer &&other) noexcept
    : cur_read_pos(other.cur_read_pos), sub_byte_counter(other.sub_byte_counter), data_cache(std::move(other.data_cache)),
      data_ptr(other.owns_data ? data_cache.data() : other.data_ptr), data_size(other.data_size),
      error(other.error), owns_data(other.owns_data) { }
    BitisDeserializer &operator=(const BitisDeserializer &other) {
        if (this != &other) {
            cur_read_pos = other.cur_read_pos; sub_byte_counter = other.sub_byte_counter;
            data_cache = other.data_cache; owns_data = other.owns_data;
            data_ptr = owns_data ? data_cache.data() : other.data_ptr;
            data_size = other.data_size; error = other.error;
        }
        return *this;
    }
    BitisDeserializer &operator=(BitisDeserializer &&other) noexcept {
        if (this != &other) {
            cur_read_pos = other.cur_read_pos; sub_byte_counter = other.sub_byte_counter;
            data_cache = std::move(other.data_cache); owns_data = other.owns_data;
            data_ptr = owns_data ? data_cache.data() : other.data_ptr;
            data_size = other.data_size; error = other.error;
        }
        return *this;
    }

    void set_error(const BitisError e) { if (error == BitisError::Ok) { error = e; } }
    bool has_error() const { return error != BitisError::Ok; }
    BitisError get_error() const { return error; }

    std::size_t consumed_bits() const { return cur_read_pos * 8 + sub_byte_counter; }
    std::size_t remaining_bits() const {
        const std::size_t total = data_size * 8;
        const std::size_t used = consumed_bits();
        return (used < total) ? total - used : 0;
    }

    // read num_bits (<= 64) bits LSB first. Returns 0 if an error is set or the input
    // is too short (-> UnexpectedEnd); nothing is consumed in that case.
    BITIS_NOINLINE uint64_t read_bits(uint8_t num_bits) {
        if (num_bits == 0 || error != BitisError::Ok) { return 0; }
        if (num_bits > 64) { set_error(BitisError::InvalidValue); return 0; }
        if (remaining_bits() < num_bits) { set_error(BitisError::UnexpectedEnd); return 0; }

        uint64_t v = 0;
        uint8_t got = 0;
        while (got < num_bits) {
            const uint8_t avail = static_cast<uint8_t>(8 - sub_byte_counter);
            const uint8_t want = static_cast<uint8_t>(num_bits - got);
            const uint8_t take = (want < avail) ? want : avail;
            const uint64_t chunk = (data_ptr[cur_read_pos] >> sub_byte_counter) & bit_masks[take];
            v |= chunk << got;
            got = static_cast<uint8_t>(got + take);
            sub_byte_counter = static_cast<uint8_t>(sub_byte_counter + take);
            if (sub_byte_counter >= 8) {
                cur_read_pos++;
                sub_byte_counter = 0;
            }
        }
        return v;
    }

    bitis_helper::BitiaDeserializerUint8Helper decode_data_uint8(const uint8_t total_bits) {
        const auto bits_to_use = std::min(total_bits, max_bitmask);
        const auto d = static_cast<uint8_t>(read_bits(bits_to_use));
        return bitis_helper::BitiaDeserializerUint8Helper{bits_to_use, d};
    }
    template<typename T>
    std::size_t decode_data(T & data, const uint8_t total_bits) {
        data = bitis_helper::from_u64_bits<T>(read_bits(total_bits));
        return total_bits;
    }
};

// min_bits / max_bits (spec section 4) of any bitis type, including generated messages
// and oneofs. bitis_max_bytes<T>() = ceil(max_bits / 8) (or BITIS_UNBOUNDED) can be used
// to size an output buffer at compile time: uint8_t buf[bitis_max_bytes<Msg>()].
template<typename T> constexpr std::size_t bitis_min_bits();
template<typename T> constexpr std::size_t bitis_max_bits();
template<typename T> constexpr std::size_t bitis_max_bytes();

namespace bitis_helper {
    // decodes into v, which has to be default constructed (nested messages, oneofs and
    // list elements are decoded in place instead of being moved through temporaries)
    template<typename T> void read_into(T &v, BitisDeserializer &des);
}

namespace bitis_helper {
    // count check before allocating (spec section 3)
    inline bool check_count(BitisDeserializer &des, const uint64_t count, std::size_t elem_min_bits) {
        if (des.has_error()) { return false; }
        if (elem_min_bits == 0) { elem_min_bits = 1; }
        if (count > des.remaining_bits() / elem_min_bits) {
            des.set_error(BitisError::UnexpectedEnd);
            return false;
        }
        return true;
    }
}

// ***************************************************************
// Shared (non-template) implementation of the scalar types. The templates only pass
// their constants; mag_bits = min(X, value bits of the C++ type), so 2^mag_bits - 1 is
// the largest valid magnitude (and the mask of the legacy truncation). The parameter
// order keeps 64 bit arguments in registers on 32 bit targets.
namespace bitis_core {
    // magnitude of uint_X / int_X (spec 2.2 / 2.3); the sign bit is written by the caller
    BITIS_NOINLINE inline void write_uint(BitisSerializer &ser, const uint8_t bits, const uint64_t mag,
                                          const uint8_t mag_bits) {
        const uint64_t max_mag = calc_bitmask(mag_bits);
        if (mag > max_mag) { ser.set_error(BitisError::ValueOutOfRange); }
        ser.write_bits(mag & max_mag, bits);
    }
    BITIS_NOINLINE inline uint64_t read_uint(BitisDeserializer &des, const uint8_t bits, const uint8_t mag_bits) {
        const uint64_t mag = des.read_bits(bits);
        if (mag > calc_bitmask(mag_bits)) {
            des.set_error(BitisError::InvalidValue);
            return 0;
        }
        return mag;
    }

    // magnitude of uint_XdY / int_XdY (spec 2.4 / 2.5); returns the written bits
    BITIS_NOINLINE inline std::size_t write_dyn(BitisSerializer &ser, const uint8_t max_bits, uint64_t mag,
                                                const uint8_t bit_packs, const uint8_t mag_bits) {
        const std::size_t start_bits = ser.total_bits;
        const uint64_t max_mag = calc_bitmask(mag_bits);
        if (mag > max_mag) {
            // legacy behaviour: truncate (but never hang)
            ser.set_error(BitisError::ValueOutOfRange);
            mag &= max_mag;
        }
        // first marker
        ser.write_bits((mag != 0) ? 1 : 0, 1);
        if (mag != 0) {
            uint8_t max_num_bits_left = max_bits;
            while (true) {
                const uint8_t cur_num_bits = std::min(max_num_bits_left, bit_packs);
                ser.write_bits(mag, cur_num_bits);
                mag = (cur_num_bits >= 64) ? 0 : (mag >> cur_num_bits);
                max_num_bits_left = static_cast<uint8_t>(max_num_bits_left - cur_num_bits);
                if (max_num_bits_left == 0) { break; }
                ser.write_bits((mag != 0) ? 1 : 0, 1);
                if (mag == 0) { break; }
            }
        }
        return ser.total_bits - start_bits;
    }
    // returns the consumed bits; mag is 0 on error
    BITIS_NOINLINE inline std::size_t read_dyn(BitisDeserializer &des, uint64_t &mag, const uint8_t max_bits,
                                               const uint8_t bit_packs, const uint8_t mag_bits) {
        const std::size_t start_bits = des.consumed_bits();
        uint64_t tval = 0;
        bool further_data_to_read = des.read_bits(1) != 0;
        uint8_t shift_bits = 0;
        uint8_t max_num_bits_left = max_bits;
        while (further_data_to_read && !des.has_error()) {
            const uint8_t cur_num_bits = std::min(max_num_bits_left, bit_packs);
            const uint64_t cdata = des.read_bits(cur_num_bits);
            tval |= cdata << shift_bits;   // shift_bits + cur_num_bits <= max_bits <= 64
            shift_bits = static_cast<uint8_t>(shift_bits + cur_num_bits);
            max_num_bits_left = static_cast<uint8_t>(max_num_bits_left - cur_num_bits);

            if (max_num_bits_left > 0) { further_data_to_read = des.read_bits(1) != 0; }
            else { further_data_to_read = false; }
        }
        if (tval > calc_bitmask(mag_bits)) {
            des.set_error(BitisError::InvalidValue);
            tval = 0;
        }
        mag = tval;
        return des.consumed_bits() - start_bits;
    }

    // enum index (spec 2.9); returns the consumed bits, id is default_id on error
    BITIS_NOINLINE inline std::size_t read_enum(BitisDeserializer &des, uint32_t &id, const uint8_t dyn_bits,
                                                const uint32_t num_variants, const uint32_t default_id) {
        uint64_t v = 0;
        const std::size_t bits = read_dyn(des, v, 32, dyn_bits, 32);
        if (!des.has_error() && v >= num_variants) {
            des.set_error(BitisError::InvalidDiscriminant);
        }
        id = des.has_error() ? default_id : static_cast<uint32_t>(v);
        return bits;
    }

    // fp_N[min, max] (spec 2.7): value -> raw wire value (NaN -> underflow)
    BITIS_NOINLINE inline uint64_t fp_to_raw(const double value, const double min_v, const double max_v,
                                             const uint8_t bits) {
        const uint64_t max_value = calc_bitmask(bits);
        if (std::isnan(value)) { return 0; }
        if (value > max_v) { return max_value; }
        if (value < min_v) { return 0; }

        // t = (v - min) / R * S + 1.0, r = round_half_away_from_zero(t)
        // (the volatile intermediate prevents FMA contraction, the spec requires
        // separately rounded operations like the Rust reference)
        const double range = max_v - min_v;
        const double steps = static_cast<double>(max_value - 2);
        const volatile double scaled = (value - min_v) / range * steps;
        const double t = scaled + 1.0;
        return static_cast<uint64_t>(std::round(t));
    }
    // raw wire value (1 .. 2^N - 2) -> value
    BITIS_NOINLINE inline double fp_from_raw(const uint64_t raw, const double min_v, const double max_v,
                                             const uint8_t bits) {
        // v = (r - 1) / S * R + min
        const double range = max_v - min_v;
        const double steps = static_cast<double>(calc_bitmask(bits) - 2);
        const volatile double scaled = static_cast<double>(raw - 1) / steps * range;
        return scaled + min_v;
    }
}


// ***************************************************************
struct BitisBool {
    bool value;

    BitisBool() : value(false) {}
    BitisBool(const bool value) : value(value) {}

    static constexpr std::size_t min_bits() { return 1; }
    static constexpr std::size_t max_bits() { return 1; }

    // ReSharper disable once CppDFAConstantFunctionResult
    std::size_t serialize(BitisSerializer &ser) const {
        ser.write_bits(value ? 1 : 0, 1);
        return 1;
    }
    static bitis_helper::BitiaDeserializerHelper<BitisBool> deserialize(BitisDeserializer &des) {
        const auto data = des.read_bits(1);
        return bitis_helper::BitiaDeserializerHelper<BitisBool>{1, BitisBool(data == 1)};
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        printf("%s", (value) ? "true" : "false" );
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const BitisBool &other) const {
        return value == other.value;
    }
    bool operator==(const BitisBool &other) const { return is_equal(other); }
    bool operator!=(const BitisBool &other) const { return !is_equal(other); }
};

// ***************************************************************
// uint_X (unsigned T) / int_X (signed T, sign-magnitude, X+1 bits)
template <typename T, uint8_t BITS>
struct IntgralWithGivenBitSize {
    static_assert(std::is_integral<T>::value, "Integral type required for IntgralWithGivenBitSize.");
    static_assert(BITS <= 64, "At most 64 bits are supported.");
    static_assert(std::numeric_limits<T>::digits + (std::is_signed<T>::value ? 1 : 0) >= BITS,
        "BITS have to be lower or equal than the number of bits of the type.");

    T value;

    IntgralWithGivenBitSize() : value() {}
    explicit IntgralWithGivenBitSize(T value) : value(value) {}
    // IntgralWithGivenBitSize(T &value) : value(value) {}

    static constexpr std::size_t min_bits() { return BITS + (std::is_signed<T>::value ? 1 : 0); }
    static constexpr std::size_t max_bits() { return min_bits(); }
    // valid range (spec 2.2 / 2.3)
    static constexpr T max_value() { return static_cast<T>(bitis_helper::max_magnitude<T, BITS>()); }
    static constexpr T min_value() {
        if constexpr (std::is_signed<T>::value) {
            return bitis_helper::from_sign_magnitude<T>(true, bitis_helper::max_magnitude<T, BITS>());
        }
        else { return T(0); }
    }
    static constexpr bool is_valid_value(const T v) {
        return bitis_helper::magnitude(v) <= bitis_helper::max_magnitude<T, BITS>();
    }
    bool is_valid() const { return is_valid_value(value); }

    std::size_t serialize(BitisSerializer &ser) const {
        if constexpr (std::is_signed<T>::value) {
            ser.write_bits((value < 0) ? 1 : 0, 1);
        }
        bitis_core::write_uint(ser, BITS, bitis_helper::magnitude(value), MAG_BITS);
        return min_bits();
    }
    static bitis_helper::BitiaDeserializerHelper<IntgralWithGivenBitSize<T, BITS>>
    deserialize(BitisDeserializer &des) {
        bool is_negative = false;
        if constexpr (std::is_signed<T>::value) {
            is_negative = des.read_bits(1) != 0;
        }
        const uint64_t mag = bitis_core::read_uint(des, BITS, MAG_BITS);
        return bitis_helper::BitiaDeserializerHelper<IntgralWithGivenBitSize>{
            min_bits(), IntgralWithGivenBitSize(bitis_helper::from_sign_magnitude<T>(is_negative, mag))
        };
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        if constexpr (std::is_signed<T>::value) {
            printf("%" PRIi64 " [int_%d]", static_cast<int64_t>(value), BITS);
        }
        else {
            printf("%" PRIu64 " [uint_%d]", static_cast<uint64_t>(value), BITS);
        }
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const IntgralWithGivenBitSize &other) const {
        return value == other.value;
    }
    bool operator==(const IntgralWithGivenBitSize& other) const { return is_equal(other); }
    bool operator!=(const IntgralWithGivenBitSize& other) const { return !is_equal(other); }

private:
    static constexpr uint8_t MAG_BITS = static_cast<uint8_t>(std::min<int>(BITS, std::numeric_limits<T>::digits));
};

template <typename T>
struct BitisFloatingPoint {
    static_assert(std::is_same<T, float>::value || std::is_same<T, double>::value,
        "float or double required for BitisFloatingPoint.");

    T value;

    BitisFloatingPoint() : value() {}
    explicit BitisFloatingPoint(T v) : value(v) { }

    static constexpr std::size_t min_bits() { return sizeof(T)*8; }
    static constexpr std::size_t max_bits() { return sizeof(T)*8; }

    std::size_t serialize(BitisSerializer &ser) const {
        ser.write_bits(bitis_helper::to_u64_bits(value), sizeof(T)*8);
        return sizeof(T)*8;
    }
    static bitis_helper::BitiaDeserializerHelper<BitisFloatingPoint> deserialize(
        BitisDeserializer &des) {
        const T val = bitis_helper::from_u64_bits<T>(des.read_bits(sizeof(T)*8));
        return bitis_helper::BitiaDeserializerHelper<BitisFloatingPoint>{
            sizeof(T)*8, BitisFloatingPoint(val)};
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        printf("%f [%s]", static_cast<double>(value), (std::is_same<T, float>::value) ? "float" : "double");
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const BitisFloatingPoint &other) const {
        return value == other.value;
    }
    bool operator==(const BitisFloatingPoint &other) const { return is_equal(other); }
    bool operator!=(const BitisFloatingPoint &other) const { return !is_equal(other); }
};

// ***************************************************************
template <typename T>
struct BitisOptional {
    typedef T ValT;
    T value;
    bool is_none;

    BitisOptional() : value(), is_none(true) {}
    BitisOptional(T val) : value(std::move(val)), is_none(false) {}
    explicit BitisOptional(T value, bool _is_none) : value(std::move(value)), is_none(_is_none) {}
    static BitisOptional create_none() { return BitisOptional(T(), true); }
    static BitisOptional create_val(T v) { return BitisOptional(std::move(v), false); }

    static constexpr std::size_t min_bits() { return 1; }
    static constexpr std::size_t max_bits() { return bitis_helper::max_bits_add(1, bitis_max_bits<T>()); }

    std::size_t serialize(BitisSerializer &ser) const {
        std::size_t num_bits = 1;
        BitisBool(!is_none).serialize(ser);
        if (!is_none) {
            num_bits += value.serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<BitisOptional> deserialize(BitisDeserializer &des) {
        const std::size_t start_bits = des.consumed_bits();
        BitisOptional r;
        r.read_into(des);
        return bitis_helper::BitiaDeserializerHelper<BitisOptional>{des.consumed_bits() - start_bits, std::move(r)};
    }
    // see bitis_helper::read_into()
    void read_into(BitisDeserializer &des) {
        is_none = des.read_bits(1) == 0;
        if (!is_none) { bitis_helper::read_into(value, des); }
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t indent=0) const {
        if (is_none) { printf("None [Optional]"); }
        else {
            value.print(indent);
            printf("[Optional]");
        }
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const BitisOptional &other) const {
        if (is_none != other.is_none) { return false; }

        if (!is_none) {
            return value.is_equal(other.value);
        }
        return true;
    }
    bool operator==(const BitisOptional &other) const { return is_equal(other); }
    bool operator!=(const BitisOptional &other) const { return !is_equal(other); }
};
// ***************************************************************
// uint_XdY / int_XdY (spec 2.4 / 2.5)
template <typename T, uint8_t MAX_BITS, uint8_t BIT_PACKS>
struct DynInteger {
    static_assert(std::is_integral<T>::value, "Integral type required for DynInteger.");
    static_assert(MAX_BITS >= 1 && MAX_BITS <= 64, "MAX_BITS has to be in 1..64.");
    static_assert(BIT_PACKS >= 1, "BIT_PACKS has to be at least 1.");
    static_assert(std::numeric_limits<T>::digits + (std::is_signed<T>::value ? 1 : 0) >= MAX_BITS,
        "MAX_BITS have to be lower or equal than the number of bits of the type.");

    T value;

    DynInteger() : value() {}
    explicit DynInteger(T value) : value(value) {}

    static constexpr std::size_t min_bits() { return std::is_signed<T>::value ? 2 : 1; }
    static constexpr std::size_t max_bits() {
        return (std::is_signed<T>::value ? 1 : 0) + bitis_helper::dyn_bits(calc_bitmask(MAX_BITS), MAX_BITS, BIT_PACKS);
    }
    static constexpr T max_value() { return static_cast<T>(bitis_helper::max_magnitude<T, MAX_BITS>()); }
    static constexpr T min_value() {
        if constexpr (std::is_signed<T>::value) {
            return bitis_helper::from_sign_magnitude<T>(true, bitis_helper::max_magnitude<T, MAX_BITS>());
        }
        else { return T(0); }
    }
    static constexpr bool is_valid_value(const T v) {
        return bitis_helper::magnitude(v) <= bitis_helper::max_magnitude<T, MAX_BITS>();
    }
    bool is_valid() const { return is_valid_value(value); }

    std::size_t serialize(BitisSerializer &ser) const {
        // indicator of the sign: send one if negative (always written for signed types)
        if constexpr (std::is_signed<T>::value) {
            ser.write_bits((value < 0) ? 1 : 0, 1);
            return 1 + bitis_core::write_dyn(ser, MAX_BITS, bitis_helper::magnitude(value), BIT_PACKS, MAG_BITS);
        }
        else {
            return bitis_core::write_dyn(ser, MAX_BITS, static_cast<uint64_t>(value), BIT_PACKS, MAG_BITS);
        }
    }
    static bitis_helper::BitiaDeserializerHelper<DynInteger> deserialize(BitisDeserializer &des) {
        uint64_t mag = 0;
        if constexpr (std::is_signed<T>::value) {
            const bool is_negative = des.read_bits(1) != 0;
            const std::size_t bits = 1 + bitis_core::read_dyn(des, mag, MAX_BITS, BIT_PACKS, MAG_BITS);
            return bitis_helper::BitiaDeserializerHelper<DynInteger>{bits,
                DynInteger(bitis_helper::from_sign_magnitude<T>(is_negative, mag))};
        }
        else {
            const std::size_t bits = bitis_core::read_dyn(des, mag, MAX_BITS, BIT_PACKS, MAG_BITS);
            return bitis_helper::BitiaDeserializerHelper<DynInteger>{bits, DynInteger(static_cast<T>(mag))};
        }
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        if constexpr (std::is_signed<T>::value) {
            printf("%" PRIi64 " [int_%dd%d]", static_cast<int64_t>(value), MAX_BITS, BIT_PACKS);
        }
        else {
            printf("%" PRIu64 " [uint_%dd%d]", static_cast<uint64_t>(value), MAX_BITS, BIT_PACKS);
        }
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const DynInteger &other) const {
        return value == other.value;
    }
    bool operator==(const DynInteger &other) const { return is_equal(other); }
    bool operator!=(const DynInteger &other) const { return !is_equal(other); }

private:
    static constexpr uint8_t MAG_BITS = static_cast<uint8_t>(std::min<int>(MAX_BITS, std::numeric_limits<T>::digits));
};

// ***************************************************************
template <typename T, std::size_t ARRAY_SIZE>
struct FixedArray {
    typedef T ValT;
    // check that T is bitis type
    //T values[ARRAY_SIZE];
    std::array<T, ARRAY_SIZE> values;

    FixedArray() : values() {}
    explicit FixedArray(const std::array<T, ARRAY_SIZE> &v) : values(v) {
        // for (std::size_t i = 0; i < ARRAY_SIZE; i++) { values[i] = v[i]; }
    }

    static constexpr std::size_t min_bits() { return ARRAY_SIZE * bitis_min_bits<T>(); }
    static constexpr std::size_t max_bits() { return bitis_helper::max_bits_mul(bitis_max_bits<T>(), ARRAY_SIZE); }

    std::size_t serialize(BitisSerializer &ser) const {
        std::size_t num_bits = 0;
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            num_bits += values[i].serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<FixedArray> deserialize(BitisDeserializer &des) {
        const std::size_t start_bits = des.consumed_bits();
        FixedArray r;
        r.read_into(des);
        return bitis_helper::BitiaDeserializerHelper<FixedArray>{des.consumed_bits() - start_bits, std::move(r)};
    }
    // see bitis_helper::read_into()
    void read_into(BitisDeserializer &des) {
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            bitis_helper::read_into(values[i], des);
            if (des.has_error()) { break; }
        }
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t indent=0) const {
        printf("[");
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            values[i].print(indent);
            if (i != ARRAY_SIZE-1) printf(", ");
        }
        printf("]{%zu}", ARRAY_SIZE);
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const FixedArray &other) const {
        for (std::size_t i = 0; i < ARRAY_SIZE; i++) {
            if (values[i] != other.values[i]) { return false; }
        }
        return true;
    }
    bool operator==(const FixedArray& other) const { return is_equal(other); }
    bool operator!=(const FixedArray& other) const { return !is_equal(other); }
};

#define ENUM_INSTANCE(x) struct x { BITIS_NAME(#x) }


// ***************************************************************
template <typename T, uint8_t DYN_BITS>
struct DynArray {
    std::vector<T> values;
    typedef T ValT;
    typedef DynInteger<uint32_t, 32, DYN_BITS> SizeT;

    DynArray() : values() { }
    explicit DynArray(const std::vector<T> &v) : values(v) { }

    static constexpr std::size_t min_bits() { return 1; }
    static constexpr std::size_t max_bits() { return BITIS_UNBOUNDED; }

    std::size_t serialize(BitisSerializer &ser) const {
        std::size_t num_bits = 0;
        if (values.size() > UINT32_MAX) { ser.set_error(BitisError::ValueOutOfRange); }
        auto data_size = SizeT(static_cast<uint32_t>(values.size()));
        num_bits += data_size.serialize(ser);
        for (std::size_t i=0; i < static_cast<uint32_t>(values.size()); i++) {
            num_bits += values[i].serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<DynArray> deserialize(BitisDeserializer &des) {
        const std::size_t start_bits = des.consumed_bits();
        DynArray r;
        r.read_into(des);
        return bitis_helper::BitiaDeserializerHelper<DynArray>{des.consumed_bits() - start_bits, std::move(r)};
    }
    // see bitis_helper::read_into(); on error values holds the complete elements
    void read_into(BitisDeserializer &des) {
        uint64_t count = 0;
        bitis_core::read_dyn(des, count, 32, DYN_BITS, 32);
        if (bitis_helper::check_count(des, count, bitis_min_bits<T>())) {
            values.reserve(static_cast<std::size_t>(count));
            for (uint64_t i=0; i < count; i++) {
                values.emplace_back();
                bitis_helper::read_into(values.back(), des);
                if (des.has_error()) { values.pop_back(); break; }
            }
        }
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t indent=0) const {
        printf("[");
        for (std::size_t i=0; i < values.size(); i++) {
            values[i].print(indent);
            if (i != values.size()-1) printf(", ");
        }
        printf("]{dyn:%zu}", values.size());
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const DynArray &other) const {
        if (values.size() != other.values.size()) { return false; }
        for (std::size_t i = 0; i < values.size(); i++) {
            if (values[i] != other.values[i]) { return false; }
        }
        return true;
    }
    bool operator==(const DynArray& other) const { return is_equal(other); }
    bool operator!=(const DynArray& other) const { return !is_equal(other); }
};

// ***************************************************************
namespace bitis_core {
    // count + bytes of binary_dY / astr_dY (spec 2.10 / 2.11); returns the written bits
    BITIS_NOINLINE inline std::size_t write_bytes(BitisSerializer &ser, const uint8_t *data, std::size_t size,
                                                  const uint8_t dyn_bits, const bool ascii_only) {
        std::size_t num_bits = 0;
        if (size > UINT32_MAX) { ser.set_error(BitisError::ValueOutOfRange); size = UINT32_MAX; }
        num_bits += write_dyn(ser, 32, size, dyn_bits, 32);

        for (std::size_t i=0; i < size; i++) {
            uint8_t cv = data[i];
            if (ascii_only && cv >= 0x80) {
                ser.set_error(BitisError::ValueOutOfRange);
                cv &= 0x7F;
            }
            ser.write_bits(cv, 8);
            num_bits += 8;
        }
        return num_bits;
    }
    // reads count + bytes into values (values is cleared first); returns consumed bits
    BITIS_NOINLINE inline std::size_t read_bytes(BitisDeserializer &des, std::vector<uint8_t> &values,
                                                 const uint8_t dyn_bits, const bool ascii_only,
                                                 const std::size_t reserve_extra) {
        const std::size_t start_bits = des.consumed_bits();
        values.clear();
        uint64_t count = 0;
        read_dyn(des, count, 32, dyn_bits, 32);
        if (bitis_helper::check_count(des, count, 8)) {
            values.reserve(static_cast<std::size_t>(count) + reserve_extra);
            for (uint64_t i=0; i < count; i++) {
                const auto cdata = static_cast<uint8_t>(des.read_bits(8));
                if (ascii_only && cdata >= 0x80) { des.set_error(BitisError::InvalidValue); }
                if (des.has_error()) { values.clear(); break; }
                values.push_back(cdata);
            }
        }
        return des.consumed_bits() - start_bits;
    }
}

template <uint8_t DYN_BITS>
struct BinaryBase {
    typedef DynInteger<uint32_t, 32, DYN_BITS> SizeT;

    BinaryBase() : values() {}
    explicit BinaryBase(const std::vector<uint8_t> &v) : values(v) {}

    std::vector<uint8_t> *get_u8_vec() { return &this->values; }
    const std::vector<uint8_t> *get_u8_vec() const { return &this->values; }

    static constexpr std::size_t min_bits() { return 1; }
    static constexpr std::size_t max_bits() { return BITIS_UNBOUNDED; }

    std::size_t serialize(BitisSerializer &ser) const {
        return serialize_bytes(ser, values.data(), values.size(), false);
    }

protected:
    static std::size_t serialize_bytes(BitisSerializer &ser, const uint8_t *data, std::size_t size,
                                       const bool ascii_only) {
        return bitis_core::write_bytes(ser, data, size, DYN_BITS, ascii_only);
    }
    // reads count + bytes into values (values is cleared first); returns consumed bits
    static std::size_t deserialize_bytes(BitisDeserializer &des, std::vector<uint8_t> &values,
                                         const bool ascii_only, const std::size_t reserve_extra) {
        return bitis_core::read_bytes(des, values, DYN_BITS, ascii_only, reserve_extra);
    }

    static bitis_helper::BitiaDeserializerHelper<BinaryBase> deserialize_base(BitisDeserializer &des) {
        BinaryBase r;
        const auto bits = deserialize_bytes(des, r.values, false, 0);
        return bitis_helper::BitiaDeserializerHelper<BinaryBase>{bits, std::move(r)};
    }

    std::vector<uint8_t> values;
};

// ***************************************************************
template <uint8_t DYN_BITS>
struct Binary : BinaryBase<DYN_BITS>{

    Binary() : BinaryBase<DYN_BITS>() {}
    explicit Binary(const std::vector<uint8_t> &v) : BinaryBase<DYN_BITS>(v) { }

    void set(const std::vector<uint8_t> &v) {
        this->values = v;
    }
    const std::vector<uint8_t> &get() const { return this->values; }
    std::size_t size() const { return this->values.size(); }

    static bitis_helper::BitiaDeserializerHelper<Binary> deserialize(BitisDeserializer &des) {
        Binary r;
        const auto bits = BinaryBase<DYN_BITS>::deserialize_bytes(des, r.values, false, 0);
        return bitis_helper::BitiaDeserializerHelper<Binary>{bits, std::move(r)};
    }

#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        printf("[");
        for (std::size_t i=0; i < this->values.size(); i++) {
            printf((i == 0) ? "%02X" : " %02X", this->values[i]);
        }
        printf("] {binary_d%u}", DYN_BITS);
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Binary &other) const {
        return this->values == other.values;
    }
    bool operator==(const Binary& other) const { return is_equal(other); }
    bool operator!=(const Binary& other) const { return !is_equal(other); }
};

// ASCII string. The internal buffer always holds a terminating NUL (not serialized),
// so get() can be used as a C string. Note: get_u8_vec() returns the buffer including
// that terminator; use size() for the string length.
template <uint8_t DYN_BITS>
struct BitisAString : BinaryBase<DYN_BITS>{

    BitisAString() : BinaryBase<DYN_BITS>() { this->values.push_back(0); }
    explicit BitisAString(const char* s) : BinaryBase<DYN_BITS>() { set(s); }
    explicit BitisAString(const std::vector<uint8_t> &v) : BinaryBase<DYN_BITS>(v) { this->values.push_back(0); }

    void set(const char* s) {
        this->values.clear();
        const std::size_t len = (s != nullptr) ? strlen(s) : 0;
        this->values.reserve(len + 1);
        for (std::size_t i = 0; i < len; i++) {
            this->values.push_back(static_cast<uint8_t>(s[i]));
        }
        this->values.push_back(0);
    }
    // NUL terminated
    const char *get() const {
        ensure_terminated();
        // ReSharper disable once CppCStyleCast
        return reinterpret_cast<const char *>(this->values.data());
    }
    const char *c_str() const { return get(); }
    // number of characters (without the terminator)
    std::size_t size() const {
        const auto &v = this->values;
        return (!v.empty() && v.back() == 0) ? v.size() - 1 : v.size();
    }
    std::size_t length() const { return size(); }

    std::size_t serialize(BitisSerializer &ser) const {
        return BinaryBase<DYN_BITS>::serialize_bytes(ser, this->values.data(), size(), true);
    }
    static bitis_helper::BitiaDeserializerHelper<BitisAString> deserialize(BitisDeserializer &des) {
        BitisAString r;
        const auto bits = BinaryBase<DYN_BITS>::deserialize_bytes(des, r.values, true, 1);
        r.values.push_back(0);
        return bitis_helper::BitiaDeserializerHelper<BitisAString>{bits, std::move(r)};
    }

#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        printf("'");
        for (std::size_t i=0; i < size(); i++) {
            printf("%c", this->values[i]);
        }
        printf("' {d%u}", DYN_BITS);
    }
#else
    void print(int16_t =0) const { }
#endif
    bool is_equal(const BitisAString &other) const {
        return size() == other.size() && std::equal(this->values.begin(), this->values.begin() + size(),
            other.values.begin());
    }
    bool operator==(const BitisAString& other) const { return is_equal(other); }
    bool operator!=(const BitisAString& other) const { return !is_equal(other); }

private:
    void ensure_terminated() const {
        // the invariant is kept by all members; only a buffer modified through
        // get_u8_vec() can lack the terminator
        auto &v = const_cast<std::vector<uint8_t>&>(this->values);
        if (v.empty() || v.back() != 0) { v.push_back(0); }
    }
};

// ***************************************************************
enum FixPrecisionMinMaxEnum {Ok, Underflow, Overflow};

namespace bitis_helper {
    // smallest unsigned type holding BITS bits
    template<uint8_t BITS> using uint_for_bits = typename std::conditional<(BITS <= 8), uint8_t,
        typename std::conditional<(BITS <= 16), uint16_t,
        typename std::conditional<(BITS <= 32), uint32_t, uint64_t>::type>::type>::type;
}

// fp_N[min, max] (spec 2.7). Stores the raw wire value: 0 = underflow, 2^N - 1 = overflow,
// otherwise the quantized value. Default: overflow. Equality compares the raw values.
template <uint8_t BITS, int64_t MIN_IVALUE, int64_t MAX_IVALUE>
struct FixPrecisionMinMax {
    static_assert(BITS >= 2 && BITS <= 64, "FixPrecisionMinMax: BITS has to be in 2..64 (spec: 2..53).");
    static_assert(MIN_IVALUE < MAX_IVALUE, "FixPrecisionMinMax: min has to be lower than max.");

    typedef bitis_helper::uint_for_bits<BITS> RawT;
    static constexpr RawT RAW_UNDERFLOW = 0;
    static constexpr RawT RAW_OVERFLOW = static_cast<RawT>(calc_bitmask(BITS));

    FixPrecisionMinMax() : raw_value(RAW_OVERFLOW) { }
    explicit FixPrecisionMinMax(const double &v) : raw_value(to_raw(v)) { }

    static constexpr std::size_t min_bits() { return BITS; }
    static constexpr std::size_t max_bits() { return BITS; }

    // stores the quantized value (or Underflow/Overflow; NaN -> Underflow)
    void set(const double &v) { raw_value = to_raw(v); }
    void set_underflow() { raw_value = RAW_UNDERFLOW; }
    void set_overflow() { raw_value = RAW_OVERFLOW; }
    // value in physical units, 0 for underflow / overflow
    double value() const {
        return has_value() ? bitis_core::fp_from_raw(raw_value, MIN_V, MAX_V, BITS) : 0.;
    }
    double get_value() const { return value(); }
    FixPrecisionMinMaxEnum state() const {
        return (raw_value == RAW_UNDERFLOW) ? Underflow : ((raw_value == RAW_OVERFLOW) ? Overflow : Ok);
    }
    FixPrecisionMinMaxEnum get_state() const { return state(); }
    bool has_value() const { return raw_value != RAW_UNDERFLOW && raw_value != RAW_OVERFLOW; }
    bool is_underflow() const { return raw_value == RAW_UNDERFLOW; }
    bool is_overflow() const { return raw_value == RAW_OVERFLOW; }
    // raw wire value (0 .. 2^N - 1)
    RawT raw() const { return raw_value; }
    void set_raw(const RawT r) { raw_value = static_cast<RawT>(r & RAW_OVERFLOW); }

    static uint64_t val_to_u64(double value, FixPrecisionMinMaxEnum state) {
        if (state == Underflow) { return RAW_UNDERFLOW; }
        if (state == Overflow) { return RAW_OVERFLOW; }
        return bitis_core::fp_to_raw(value, MIN_V, MAX_V, BITS);
    }
    static void u64_to_val(uint64_t vu, double &value, FixPrecisionMinMaxEnum &state) {
        if(vu == RAW_UNDERFLOW) { state = Underflow; }
        else if(vu == RAW_OVERFLOW) { state = Overflow; }
        else {
            state = Ok;
            value = bitis_core::fp_from_raw(vu, MIN_V, MAX_V, BITS);
        }
    }

    std::size_t serialize(BitisSerializer &ser) const {
        ser.write_bits(raw_value, BITS);
        return BITS;
    }
    static bitis_helper::BitiaDeserializerHelper<FixPrecisionMinMax> deserialize(BitisDeserializer &des) {
        FixPrecisionMinMax d;
        d.raw_value = static_cast<RawT>(des.read_bits(BITS));
        return bitis_helper::BitiaDeserializerHelper<FixPrecisionMinMax>{BITS, d};
    }
#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        if (is_underflow()) {
            printf("Underflow");
        }
        else if (is_overflow()) {
            printf("Overflow");
        }
        else {
            printf("%f", value());
        }
        printf(" [FP[%" PRIi64 ", %" PRIi64 "]]", MIN_IVALUE, MAX_IVALUE);
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const FixPrecisionMinMax &other) const { return raw_value == other.raw_value; }
    bool operator==(const FixPrecisionMinMax &other) const { return is_equal(other); }
    bool operator!=(const FixPrecisionMinMax &other) const { return !is_equal(other); }

private:
    static constexpr double MIN_V = static_cast<double>(MIN_IVALUE);
    static constexpr double MAX_V = static_cast<double>(MAX_IVALUE);
    static RawT to_raw(const double &v) { return static_cast<RawT>(bitis_core::fp_to_raw(v, MIN_V, MAX_V, BITS)); }

    RawT raw_value;
};

// ***************************************************************
// ***************************************************************
namespace bitis_helper {
    template<typename ...Args> struct Collector{ };

    template<typename ...> struct CollectorSize;
    template<typename ... Ts> struct CollectorSize<Collector<Ts...>> {
        static constexpr std::size_t value = sizeof...(Ts);
    };

    template<typename T, uint32_t IDX> struct ETN {
        typedef T type;
        typedef std::integral_constant<uint32_t, IDX> id;
    };

    // ***
    template<typename ...> struct EnumeratedListImpl;
    template<typename T, uint32_t CIDX, typename ... ETs, typename TT, typename ... TTs>
    struct EnumeratedListImpl<bitis_helper::Collector<ETs...>, ETN<T, CIDX>, TT, TTs...> {
        typedef typename EnumeratedListImpl<bitis_helper::Collector<ETs..., ETN<T, CIDX>>,
            ETN<TT, CIDX+1>, TTs...>::type type;
    };
    template<typename T, uint32_t CIDX, typename ... ETs>
    struct EnumeratedListImpl<bitis_helper::Collector<ETs...>, ETN<T, CIDX>> {
        typedef bitis_helper::Collector<ETs..., ETN<T, CIDX>> type;
    };

    template<typename ... > struct EnumeratedList;
    template<typename T, typename ... Ts>
    struct EnumeratedList<T, Ts ...> {
        typedef typename EnumeratedListImpl<bitis_helper::Collector<>, ETN<T, 0>, Ts...>::type type;
    };

    template<typename ... > struct EnumeratedListCollector;
    template<typename T, typename ... Ts>
    struct EnumeratedListCollector<bitis_helper::Collector<T, Ts ...>> {
        typedef typename EnumeratedListImpl<bitis_helper::Collector<>, ETN<T, 0>, Ts...>::type type;
    };

    template<typename ...> struct ContainsType;
    template<typename U, typename T> struct ContainsType<U, bitis_helper::Collector<T>> { static constexpr auto value = false; };
    template<typename U> struct ContainsType<U, bitis_helper::Collector<U>> { static constexpr auto value = true; };
    template<typename U, typename... Ts> struct ContainsType<U, bitis_helper::Collector<U, Ts...>> {
        static constexpr auto value = true;
    };
    template<typename U, typename T, typename... Ts> struct ContainsType<U, bitis_helper::Collector<T, Ts...>> {
        static constexpr auto value = ContainsType<U, bitis_helper::Collector<Ts...>>::value;
    };
}

// ***************************************************************
namespace bitis_enum_helper {
    using namespace bitis_helper;

    template<typename ...> struct EnumID_Impl;
    template<typename SEL_ENUM, typename T, typename ... Ts>
    struct EnumID_Impl<SEL_ENUM, bitis_helper::Collector<T, Ts ...>> {
        static constexpr size_t get_id() {
            return EnumID_Impl<SEL_ENUM, bitis_helper::Collector<Ts ...>>::get_id();
        }
    };
    template<typename T, typename ... Ts>
    struct EnumID_Impl<typename T::type, bitis_helper::Collector<T, Ts ...>> {
        static constexpr size_t get_id() { return T::id::value; }
    };
    template<typename SEL_ENUM>
    struct EnumID_Impl<SEL_ENUM, bitis_helper::Collector<>> {
        // ReSharper disable once CppStaticAssertFailure
        static_assert(bitis_helper::dependent_false<SEL_ENUM>::value, "Enum unknown");
        static constexpr size_t get_id() { return 0; }
    };
    template<typename ...> struct EnumName_Impl;
    template<typename T, typename ... Ts>
    struct EnumName_Impl<bitis_helper::Collector<T, Ts ...>> {
        static const char *get_name(uint32_t enum_id) {
            if (enum_id == T::id::value) {
                return T::type::name;
            }
            return EnumName_Impl<bitis_helper::Collector<Ts ...>>::get_name(enum_id);
        }
    };
    template<>
    struct EnumName_Impl<bitis_helper::Collector<>> {
        static const char *get_name(uint32_t) { return nullptr; }
    };

    template<typename ENUM_TO_FIND, typename ENUMERATED_ENUM_COL>
    constexpr uint32_t get_id() {
        return static_cast<uint32_t>(EnumID_Impl<ENUM_TO_FIND, ENUMERATED_ENUM_COL>::get_id());
    }
    template<typename ENUMERATED_ENUM_COL>
    const char *get_name(uint32_t enum_id) {
        return EnumName_Impl<ENUMERATED_ENUM_COL>::get_name(enum_id);
    }
}


// template<typename ...> struct BitisEnum;
template<typename ES_COLLECTOR, typename DEFAULT_ENUM, uint8_t DYN_BITS>
struct BitisEnum {
    DynInteger<uint32_t, 32, DYN_BITS> value;

    typedef ES_COLLECTOR EnumCollector;
    typedef typename bitis_helper::EnumeratedListCollector<ES_COLLECTOR>::type EnumeratedEnumCollector;

    static_assert(bitis_helper::ContainsType<DEFAULT_ENUM, EnumCollector>::value,
        "The default enum value has to be one of the enum values.");

    BitisEnum() : value(bitis_enum_helper::get_id<DEFAULT_ENUM, EnumeratedEnumCollector>()) {}

private:
    explicit BitisEnum(uint32_t enum_id) : value(enum_id) {}

public:
    static constexpr uint32_t num_variants() {
        return static_cast<uint32_t>(bitis_helper::CollectorSize<ES_COLLECTOR>::value);
    }
    static constexpr std::size_t min_bits() { return 1; }
    static constexpr std::size_t max_bits() { return bitis_helper::dyn_bits(num_variants() - 1, 32, DYN_BITS); }

    template<typename SET_ENUM>
    static BitisEnum create_enum() {
        static_assert(bitis_helper::ContainsType<SET_ENUM, EnumCollector>::value, "Unknown enum value.");
        auto id_in = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
        return BitisEnum(id_in);
    }

    template<typename SET_ENUM>
    void set_enum() {
        static_assert(bitis_helper::ContainsType<SET_ENUM, EnumCollector>::value, "Unknown enum value.");
        this->value.value = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
    }
    template<typename SET_ENUM>
    bool is_enum() const {
        static_assert(bitis_helper::ContainsType<SET_ENUM, EnumCollector>::value, "Unknown enum value.");
        auto id_in = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
        return this->value.value == id_in;
    }
    bool is_valid() const { return value.value < num_variants(); }

    std::size_t serialize(BitisSerializer &ser) const {
        if (!is_valid()) { ser.set_error(BitisError::ValueOutOfRange); }
        return value.serialize(ser);
    }
    static bitis_helper::BitiaDeserializerHelper<BitisEnum> deserialize(BitisDeserializer &des) {
        uint32_t id = 0;
        const std::size_t bits = bitis_core::read_enum(des, id, DYN_BITS, num_variants(),
            bitis_enum_helper::get_id<DEFAULT_ENUM, EnumeratedEnumCollector>());
        return bitis_helper::BitiaDeserializerHelper<BitisEnum>{bits, BitisEnum(id)};
    }

#ifndef BITIS_NO_PRINT
    void print(int16_t /*indent*/=0) const {
        auto v = bitis_enum_helper::get_name<EnumeratedEnumCollector>(value.value);
        // ReSharper disable once CppCStyleCast
        printf("%s (id:%" PRIu32 ") [enum_%u]", v ? v : "?", value.value, DYN_BITS);
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const BitisEnum &other) const {
        return this->value == other.value;
    }
    bool operator==(const BitisEnum &other) const { return is_equal(other); }
    bool operator!=(const BitisEnum &other) const { return !is_equal(other); }
};



namespace message_helper{
    using namespace bitis_helper;

    // ***
    template<typename ...> struct MessageAttribute;
    template<typename T, uint32_t IDX> struct MessageAttribute<ETN<T, IDX>> {
        typedef std::integral_constant<uint32_t, IDX> id;
        typedef T type;
        T value;
    };

    template<typename ...> struct MessageTE;
    template<> struct MessageTE<> { };
    template<typename ... Ts>
    struct MessageTE<bitis_helper::Collector<Ts ...>> : MessageAttribute<Ts> ... { };

    template<typename ...> struct MessageT;
    template<> struct MessageT<> : MessageTE<> { typedef bitis_helper::Collector<> AttrList; };
    template<typename ... Ts>
    struct MessageT : MessageTE<typename EnumeratedList<Ts...>::type> {
        typedef typename EnumeratedList<Ts...>::type AttrList;
    };

    template<typename ...> struct MessageMinBits;
    template<typename ... Ts> struct MessageMinBits<MessageT<Ts...>> {
        static constexpr std::size_t value() { return (static_cast<std::size_t>(0) + ... + bitis_min_bits<Ts>()); }
    };
    template<typename ...> struct MessageMaxBits;
    template<typename ... Ts> struct MessageMaxBits<MessageT<Ts...>> {
        static constexpr std::size_t value() {
            std::size_t r = 0;
            ((r = max_bits_add(r, bitis_max_bits<Ts>())), ...);
            return r;
        }
    };

#ifndef BITIS_NO_PRINT
    inline void print_attr_begin(const int16_t indent, const char *name) {
        print_indent(indent);
        printf("%s: ", name);
    }
    inline void print_attr_end(const int16_t indent) { printf((indent >= 0) ? ",\n" : ", "); }
#endif

    // the attributes of a message: ETs = ETN<attribute type, index>... (one function per
    // message and operation)
    template<typename ...> struct MessageFold;
    template<typename ... ETs> struct MessageFold<bitis_helper::Collector<ETs ...>> {
        template<typename Msg_STRUCT>
        static void serialize(const Msg_STRUCT *d, BitisSerializer &ser) {
            (static_cast<const MessageAttribute<ETs>*>(d)->value.serialize(ser), ...);
            (void) d; (void) ser;
        }
        template<typename Msg_STRUCT>
        static void read_into(Msg_STRUCT *d, BitisDeserializer &des) {
            (bitis_helper::read_into(static_cast<MessageAttribute<ETs>*>(d)->value, des), ...);
            (void) d; (void) des;
        }
#ifndef BITIS_NO_PRINT
        template<typename Msg_STRUCT>
        static void print(const Msg_STRUCT *d, int16_t indent, const char * const *attr_names) {
            ((print_attr_begin(indent, attr_names[ETs::id::value]),
              static_cast<const MessageAttribute<ETs>*>(d)->value.print(indent),
              print_attr_end(indent)), ...);
            (void) d; (void) indent; (void) attr_names;
        }
#endif
    };

    // Note: the attributes are accessed through MsgT, which has to have the same layout
    // as the generated message struct (attributes first, in declaration order).
    template<typename Msg>
    size_t msg_serialize(const Msg *d, BitisSerializer &ser) {
        typedef typename std::remove_const<Msg>::type MsgNC;
        auto *dt = reinterpret_cast<const typename MsgNC::MsgT*>(d);

        const std::size_t start_bits = ser.total_bits;
        MessageFold<typename MsgNC::MsgT::AttrList>::serialize(dt, ser);
        return ser.total_bits - start_bits;
    }
    // decodes into a default constructed message
    template<typename Msg>
    void msg_read_into(Msg &m, BitisDeserializer &des) {
        MessageFold<typename Msg::MsgT::AttrList>::read_into(reinterpret_cast<typename Msg::MsgT*>(&m), des);
    }
    template<typename Msg>
    bitis_helper::BitiaDeserializerHelper<Msg> msg_deserialize(BitisDeserializer &des) {
        const std::size_t start_bits = des.consumed_bits();
        Msg buffer;
        msg_read_into(buffer, des);
        return bitis_helper::BitiaDeserializerHelper<Msg>{des.consumed_bits() - start_bits, std::move(buffer)};
    }
#ifndef BITIS_NO_PRINT
    template<typename Msg>
    void msg_print(const Msg *d, int16_t indent, const char * const *attr_names) {
        typedef typename std::remove_const<Msg>::type MsgNC;
        MessageFold<typename MsgNC::MsgT::AttrList>::print(reinterpret_cast<const typename MsgNC::MsgT*>(d),
            indent, attr_names);
    }
#else
    template<typename Msg>
    void msg_print(const Msg *, int16_t, const char * const *) { }
#endif
    template<typename Msg>
    constexpr std::size_t msg_min_bits() {
        return MessageMinBits<typename Msg::MsgT>::value();
    }
    template<typename Msg>
    constexpr std::size_t msg_max_bits() {
        return MessageMaxBits<typename Msg::MsgT>::value();
    }
}

namespace oneof_helper {
    using namespace bitis_helper;

    template<typename...> struct MaxSizeof;
    template<typename T> struct MaxSizeof<T> {
        static constexpr size_t value = sizeof(T);
    };
    template<typename T, typename... Ts> struct MaxSizeof<T, Ts...> {
      static constexpr size_t value = std::max(sizeof(T), MaxSizeof<Ts...>::value);
    };
    template<typename...> struct MaxAlignof;
    template<typename T> struct MaxAlignof<T> {
        static constexpr size_t value = alignof(T);
    };
    template<typename T, typename... Ts> struct MaxAlignof<T, Ts...> {
      static constexpr size_t value = std::max(alignof(T), MaxAlignof<Ts...>::value);
    };

    // Tagged storage for the value of a oneof. Holds at most one object of one of
    // the types Ts (properly aligned, constructed, copied, moved and destroyed).
    // get<T>() of a type that is not stored (e.g. a default constructed oneof)
    // replaces the content by a default constructed T (that is why the storage is
    // mutable): a oneof therefore always has a value for its selected variant.
    template<typename ... Ts>
    struct UnionT {
        static_assert(sizeof...(Ts) > 0, "UnionT needs at least one type");
        static_assert(sizeof...(Ts) < 255, "UnionT supports at most 254 types");
    private:
        static constexpr uint8_t EMPTY = 0;

        alignas(MaxAlignof<Ts...>::value) mutable unsigned char data[MaxSizeof<Ts ...>::value];
        mutable uint8_t active;   // 0: empty, otherwise index of the stored type + 1

        template<std::size_t I> using TypeAt = typename std::tuple_element<I, std::tuple<Ts...>>::type;

        template<typename T>
        static constexpr uint8_t index_of() {
            constexpr bool matches[] = {std::is_same<T, Ts>::value...};
            for (std::size_t i = 0; i < sizeof...(Ts); i++) {
                if (matches[i]) { return static_cast<uint8_t>(i); }
            }
            return static_cast<uint8_t>(sizeof...(Ts));
        }
        template<typename T> T *ptr() const { return std::launder(reinterpret_cast<T*>(data)); }

        template<typename T> static void destroy_obj(T *p) { p->~T(); }
        template<std::size_t ... Is>
        void destroy(std::index_sequence<Is...>) const {
            ((active == Is + 1 ? destroy_obj(ptr<TypeAt<Is>>()) : (void) 0), ...);
        }
        template<std::size_t ... Is>
        void copy_from(const UnionT &other, std::index_sequence<Is...>) {
            ((other.active == Is + 1 ? (void) (::new (static_cast<void*>(data)) TypeAt<Is>(*other.template ptr<TypeAt<Is>>()), active = static_cast<uint8_t>(Is + 1))
                : (void) 0), ...);
        }
        template<std::size_t ... Is>
        void move_from(UnionT &other, std::index_sequence<Is...>) {
            ((other.active == Is + 1 ? (void) (::new (static_cast<void*>(data)) TypeAt<Is>(std::move(*other.template ptr<TypeAt<Is>>())), active = static_cast<uint8_t>(Is + 1))
                : (void) 0), ...);
        }
        void reset_impl() const {
            destroy(std::index_sequence_for<Ts...>{});
            active = EMPTY;
        }

    public:
        UnionT() noexcept : active(EMPTY) { }
        UnionT(const UnionT &other) : active(EMPTY) { copy_from(other, std::index_sequence_for<Ts...>{}); }
        UnionT(UnionT &&other) noexcept((std::is_nothrow_move_constructible<Ts>::value && ...)) : active(EMPTY) {
            move_from(other, std::index_sequence_for<Ts...>{});
        }
        UnionT &operator=(const UnionT &other) {
            if (this != &other) {
                if (other.active == EMPTY) { reset_impl(); }
                else {
                    UnionT tmp(other);   // other may be part of the stored value
                    reset_impl();
                    move_from(tmp, std::index_sequence_for<Ts...>{});
                }
            }
            return *this;
        }
        UnionT &operator=(UnionT &&other) noexcept((std::is_nothrow_move_constructible<Ts>::value && ...)) {
            if (this != &other) {
                reset_impl();
                move_from(other, std::index_sequence_for<Ts...>{});
            }
            return *this;
        }
        ~UnionT() { reset_impl(); }

        void reset() { reset_impl(); }
        bool has_value() const { return active != EMPTY; }
        template<typename T>
        bool holds() const {
            static_assert(ContainsType<T, bitis_helper::Collector<Ts...>>::value, "Type not part of the oneof");
            return active == index_of<T>() + 1;
        }

        template<typename T, typename ... Args>
        T &emplace(Args && ... args) {
            static_assert(ContainsType<T, bitis_helper::Collector<Ts...>>::value, "Type not part of the oneof");
            if constexpr (sizeof...(Args) == 0) {
                reset_impl();
                ::new (static_cast<void*>(data)) T();
            }
            else {
                T tmp(std::forward<Args>(args)...);   // args may refer to the stored value
                reset_impl();
                ::new (static_cast<void*>(data)) T(std::move(tmp));
            }
            active = static_cast<uint8_t>(index_of<T>() + 1);
            return *ptr<T>();
        }

        template<typename V>
        void set(V &&v) {
            typedef typename std::decay<V>::type T;
            static_assert(ContainsType<T, bitis_helper::Collector<Ts...>>::value, "Type not part of the oneof");
            if (holds<T>()) { *ptr<T>() = std::forward<V>(v); }
            else { emplace<T>(std::forward<V>(v)); }
        }
        template<typename T>
        T *get() const {
            static_assert(ContainsType<T, bitis_helper::Collector<Ts...>>::value, "Type not part of the oneof");
            if (!holds<T>()) {
                reset_impl();
                ::new (static_cast<void*>(data)) T();
                active = static_cast<uint8_t>(index_of<T>() + 1);
            }
            return ptr<T>();
        }
    };

    template<typename ...> struct OneOfMinBits;
    template<typename T> struct OneOfMinBits<bitis_helper::Collector<T>> {
        static constexpr std::size_t value() { return bitis_min_bits<typename T::OOType>(); }
    };
    template<typename T, typename ... Ts> struct OneOfMinBits<bitis_helper::Collector<T, Ts ...>> {
        static constexpr std::size_t value() {
            return std::min(bitis_min_bits<typename T::OOType>(), OneOfMinBits<bitis_helper::Collector<Ts...>>::value());
        }
    };

    template<typename ...> struct OneOfMaxBits;
    template<typename T> struct OneOfMaxBits<bitis_helper::Collector<T>> {
        static constexpr std::size_t value() { return bitis_max_bits<typename T::OOType>(); }
    };
    template<typename T, typename ... Ts> struct OneOfMaxBits<bitis_helper::Collector<T, Ts ...>> {
        static constexpr std::size_t value() {
            return std::max(bitis_max_bits<typename T::OOType>(), OneOfMaxBits<bitis_helper::Collector<Ts...>>::value());
        }
    };

    // the elements of a oneof (one function per oneof and operation); the selector
    // value is the index into Ts
    template<typename ...> struct OneOfFold;
    template<typename OOT_STRUCT, typename ... Ts>
    struct OneOfFold<OOT_STRUCT, bitis_helper::Collector<Ts ...>> {
        template<std::size_t I> using OOType = typename std::tuple_element<I, std::tuple<typename Ts::OOType...>>::type;
        typedef std::index_sequence_for<Ts...> Idx;

        static std::size_t serialize(const OOT_STRUCT *d, BitisSerializer &ser) {
            const std::size_t start_bits = ser.total_bits;
            d->oo_selector.serialize(ser);
            if (!write_value(d, ser, Idx{})) { ser.set_error(BitisError::ValueOutOfRange); }   // invalid selector
            return ser.total_bits - start_bits;
        }
        // d has to be default constructed (selector: default element, no value)
        static void read_into(OOT_STRUCT &d, BitisDeserializer &des) {
            d.oo_selector = OOT_STRUCT::T_OOEnum::deserialize(des).data;   // default on error
            if (!des.has_error() && !read_value(d, des, Idx{})) { des.set_error(BitisError::InvalidDiscriminant); }
        }
#ifndef BITIS_NO_PRINT
        static void print(const OOT_STRUCT *d, int16_t indent) {
            d->oo_selector.print(indent);
            printf(" -> ");
            if (!print_value(d, indent, Idx{})) { printf("?"); }
        }
#endif
        static bool is_equal(const OOT_STRUCT *self, const OOT_STRUCT *other) {
            if (self->oo_selector.value.value != other->oo_selector.value.value) { return false; }
            return equal_value(self, other, Idx{});
        }

    private:
        template<std::size_t ... Is>
        static bool write_value(const OOT_STRUCT *d, BitisSerializer &ser, std::index_sequence<Is...>) {
            const uint32_t sel = d->oo_selector.value.value;
            return ((sel == Is && (d->oo_value.template get<OOType<Is>>()->serialize(ser), true)) || ...);
        }
        template<std::size_t ... Is>
        static bool read_value(OOT_STRUCT &d, BitisDeserializer &des, std::index_sequence<Is...>) {
            const uint32_t sel = d.oo_selector.value.value;
            return ((sel == Is && (bitis_helper::read_into(d.oo_value.template emplace<OOType<Is>>(), des), true)) || ...);
        }
#ifndef BITIS_NO_PRINT
        template<std::size_t ... Is>
        static bool print_value(const OOT_STRUCT *d, int16_t indent, std::index_sequence<Is...>) {
            const uint32_t sel = d->oo_selector.value.value;
            return ((sel == Is && (d->oo_value.template get<OOType<Is>>()->print(indent), true)) || ...);
        }
#endif
        template<std::size_t ... Is>
        static bool equal_value(const OOT_STRUCT *self, const OOT_STRUCT *other, std::index_sequence<Is...>) {
            const uint32_t sel = self->oo_selector.value.value;
            bool r = false;
            ((sel == Is && (r = self->oo_value.template get<OOType<Is>>()->is_equal(
                *other->oo_value.template get<OOType<Is>>()), true)) || ...);
            return r;
        }
    };
    template<typename OOTS> using OneOfFoldFor = OneOfFold<typename std::remove_const<OOTS>::type,
        typename std::remove_const<OOTS>::type::T_OOEnum::EnumCollector>;

    template<typename OOTS>
    size_t oneof_serialize(const OOTS *d, BitisSerializer &ser) {
        return OneOfFoldFor<OOTS>::serialize(d, ser);
    }

    // decodes into a default constructed oneof
    template<typename OOTS>
    void oneof_read_into(OOTS &d, BitisDeserializer &des) {
        OneOfFoldFor<OOTS>::read_into(d, des);
    }
    template<typename OOTS>
    BitiaDeserializerHelper<OOTS> oneof_deserialize(BitisDeserializer &des) {
        const std::size_t start_bits = des.consumed_bits();
        OOTS d;
        oneof_read_into(d, des);
        return BitiaDeserializerHelper<OOTS>{des.consumed_bits() - start_bits, std::move(d)};
    }

#ifndef BITIS_NO_PRINT
    template<typename OOTS>
    void oneof_print(const OOTS *d, int16_t indent) {
        OneOfFoldFor<OOTS>::print(d, indent);
    }
#else
    template<typename OOTS>
    void oneof_print(const OOTS *, int16_t) { }
#endif

    template<typename OOTS>
    bool oneof_is_equal(const OOTS *self, const OOTS *other) {
        return OneOfFoldFor<OOTS>::is_equal(self, other);
    }
    template<typename OOTS>
    constexpr std::size_t oneof_min_bits() {
        return 1 + OneOfMinBits<typename OOTS::T_OOEnum::EnumCollector>::value();
    }
    template<typename OOTS>
    constexpr std::size_t oneof_max_bits() {
        return max_bits_add(OOTS::T_OOEnum::max_bits(), OneOfMaxBits<typename OOTS::T_OOEnum::EnumCollector>::value());
    }
}

// ***************************************************************
namespace bitis_helper {
    template<typename T, typename = void> struct HasMinBits : std::false_type { };
    template<typename T> struct HasMinBits<T, std::void_t<decltype(T::min_bits())>> : std::true_type { };
    template<typename T, typename = void> struct HasMaxBits : std::false_type { };
    template<typename T> struct HasMaxBits<T, std::void_t<decltype(T::max_bits())>> : std::true_type { };
    template<typename T, typename = void> struct IsMessage : std::false_type { };
    template<typename T> struct IsMessage<T, std::void_t<typename T::MsgT>> : std::true_type { };
    template<typename T, typename = void> struct IsOneOf : std::false_type { };
    template<typename T> struct IsOneOf<T, std::void_t<typename T::T_OOEnum>> : std::true_type { };
}

template<typename T> constexpr std::size_t bitis_min_bits() {
    if constexpr (bitis_helper::HasMinBits<T>::value) { return T::min_bits(); }
    else if constexpr (bitis_helper::IsMessage<T>::value) { return message_helper::msg_min_bits<T>(); }
    else if constexpr (bitis_helper::IsOneOf<T>::value) { return oneof_helper::oneof_min_bits<T>(); }
    else {
        static_assert(bitis_helper::dependent_false<T>::value, "bitis_min_bits(): not a bitis type");
        return 0;
    }
}
template<typename T> constexpr std::size_t bitis_max_bits() {
    if constexpr (bitis_helper::HasMaxBits<T>::value) { return T::max_bits(); }
    else if constexpr (bitis_helper::IsMessage<T>::value) { return message_helper::msg_max_bits<T>(); }
    else if constexpr (bitis_helper::IsOneOf<T>::value) { return oneof_helper::oneof_max_bits<T>(); }
    else {
        static_assert(bitis_helper::dependent_false<T>::value, "bitis_max_bits(): not a bitis type");
        return 0;
    }
}
namespace bitis_helper {
    template<typename T, typename = void> struct HasReadInto : std::false_type { };
    template<typename T> struct HasReadInto<T, std::void_t<decltype(std::declval<T&>().read_into(
        std::declval<BitisDeserializer&>()))>> : std::true_type { };

    template<typename T> void read_into(T &v, BitisDeserializer &des) {
        if constexpr (IsMessage<T>::value) { message_helper::msg_read_into(v, des); }
        else if constexpr (IsOneOf<T>::value) { oneof_helper::oneof_read_into(v, des); }
        else if constexpr (HasReadInto<T>::value) { v.read_into(des); }
        else { v = T::deserialize(des).data; }
    }
}
template<typename T> constexpr std::size_t bitis_max_bytes() {
    constexpr std::size_t bits = bitis_max_bits<T>();
    return (bits == BITIS_UNBOUNDED) ? BITIS_UNBOUNDED : bits / 8 + ((bits % 8 != 0) ? 1 : 0);
}

// ***************************************************************
// Legacy API: serialize truncates out-of-range values silently.
template<typename T>
std::vector<uint8_t> serialize(const T & msg) {
    auto ser = BitisSerializer();
    msg.serialize(ser);
    ser.finalize();
    return std::move(ser.data_cache);
}

// Legacy API: on any decoding error {bits = 0, data = T()} is returned.
template<typename T>
bitis_helper::BitiaDeserializerHelper<T> deserialize_with_size(const std::vector<uint8_t>& data) {
    auto des = BitisDeserializer(data.data(), data.size());
    auto dd = T::deserialize(des);
    if (des.has_error()) {
        return bitis_helper::BitiaDeserializerHelper<T>{0, T()};
    }
    return dd;
}
// Legacy API: on any decoding error T() is returned.
template<typename T>
T deserialize(const std::vector<uint8_t>& data) {
    return deserialize_with_size<T>(data).data;
}

// ***************************************************************
// Fallible API (bitis 0.11)

// Encodes msg into out (replacing its content). Returns ValueOutOfRange (and clears
// out) if a value does not fit its declared type.
template<typename T>
BitisError try_serialize(const T &msg, std::vector<uint8_t> &out, BitisSize *size = nullptr) {
    auto ser = BitisSerializer();
    msg.serialize(ser);
    const auto s = ser.finalize();
    if (ser.has_error()) {
        out.clear();
        return ser.error;
    }
    out = std::move(ser.data_cache);
    if (size) { *size = s; }
    return BitisError::Ok;
}

// Encodes msg into buf (cap bytes). Returns BufferTooSmall if the message needs more
// than cap bytes (size then receives the needed size; the content of buf is undefined)
// and ValueOutOfRange if a value does not fit its declared type. The bit stream is the
// same as with the vector variant; bitis_max_bytes<T>() bytes always suffice.
template<typename T>
BitisError try_serialize(const T &msg, uint8_t *buf, const std::size_t cap, BitisSize *size = nullptr) {
    BitisSerializer ser(buf, cap);
    msg.serialize(ser);
    const auto s = ser.finalize();
    if (size && (!ser.has_error() || ser.error == BitisError::BufferTooSmall)) { *size = s; }
    return ser.error;
}

// Decodes a T from the start of data. Trailing bytes are ignored; size receives the
// consumed bits / bytes. out is only modified on success.
template<typename T>
BitisError try_deserialize(const uint8_t *data, const std::size_t len, T &out, BitisSize *size = nullptr) {
    auto des = BitisDeserializer(data, len);
    auto r = T::deserialize(des);
    if (des.has_error()) { return des.get_error(); }
    out = std::move(r.data);
    if (size) {
        const std::size_t bits = des.consumed_bits();
        *size = BitisSize{bits, (bits + 7) / 8};
    }
    return BitisError::Ok;
}
template<typename T>
BitisError try_deserialize(const std::vector<uint8_t> &data, T &out, BitisSize *size = nullptr) {
    return try_deserialize<T>(data.data(), data.size(), out, size);
}

// Like try_deserialize() but fails with TrailingData if data is longer than the message.
template<typename T>
BitisError try_deserialize_exact(const uint8_t *data, const std::size_t len, T &out, BitisSize *size = nullptr) {
    auto des = BitisDeserializer(data, len);
    auto r = T::deserialize(des);
    if (des.has_error()) { return des.get_error(); }
    const std::size_t bits = des.consumed_bits();
    if ((bits + 7) / 8 != len) { return BitisError::TrailingData; }
    out = std::move(r.data);
    if (size) { *size = BitisSize{bits, (bits + 7) / 8}; }
    return BitisError::Ok;
}
template<typename T>
BitisError try_deserialize_exact(const std::vector<uint8_t> &data, T &out, BitisSize *size = nullptr) {
    return try_deserialize_exact<T>(data.data(), data.size(), out, size);
}

#endif // BITIS_LIB_H
