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


inline void print_indent(const int16_t indent) {
    for(int i=0; i < indent; i++) { printf(" "); }
}

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
};

inline const char *bitis_error_name(const BitisError e) {
    switch (e) {
        case BitisError::Ok: return "Ok";
        case BitisError::UnexpectedEnd: return "UnexpectedEnd";
        case BitisError::InvalidValue: return "InvalidValue";
        case BitisError::InvalidDiscriminant: return "InvalidDiscriminant";
        case BitisError::TrailingData: return "TrailingData";
        case BitisError::ValueOutOfRange: return "ValueOutOfRange";
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
struct BitisSerializer {
    uint16_t cur_cache_u8 = 0;
    uint8_t sub_byte_counter = 0;
    std::vector<uint8_t> data_cache;
    std::size_t final_total_bits = 0;
    std::size_t total_bits = 0;
    BitisError error = BitisError::Ok;

    void set_error(const BitisError e) { if (error == BitisError::Ok) { error = e; } }
    bool has_error() const { return error != BitisError::Ok; }

    // write the low num_bits (<= 64) bits of v, LSB first
    void write_bits(uint64_t v, uint8_t num_bits) {
        if (num_bits > 64) { num_bits = 64; }
        total_bits += num_bits;
        while (num_bits > 0) {
            const uint8_t space = static_cast<uint8_t>(8 - sub_byte_counter);
            const uint8_t take = (num_bits < space) ? num_bits : space;
            cur_cache_u8 = static_cast<uint16_t>(cur_cache_u8 | ((v & bit_masks[take]) << sub_byte_counter));
            sub_byte_counter = static_cast<uint8_t>(sub_byte_counter + take);
            num_bits = static_cast<uint8_t>(num_bits - take);
            v >>= take;
            if (sub_byte_counter >= 8) {
                data_cache.push_back(static_cast<uint8_t>(cur_cache_u8 & 255));
                sub_byte_counter = 0;
                cur_cache_u8 = 0;
            }
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
        if (sub_byte_counter > 0) {
            data_cache.push_back(static_cast<uint8_t>(cur_cache_u8 & 255));
            sub_byte_counter = 0;
            cur_cache_u8 = 0;
        }
        final_total_bits = total_bits;
        return BitisSize{final_total_bits, data_cache.size()};
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
    uint64_t read_bits(uint8_t num_bits) {
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

// min_bits (spec section 4) of any bitis type, including generated messages and oneofs
template<typename T> constexpr std::size_t bitis_min_bits();

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
struct BitisBool {
    bool value;

    BitisBool() : value(false) {}
    BitisBool(const bool value) : value(value) {}

    static constexpr std::size_t min_bits() { return 1; }

    // ReSharper disable once CppDFAConstantFunctionResult
    std::size_t serialize(BitisSerializer &ser) const {
        ser.write_bits(value ? 1 : 0, 1);
        return 1;
    }
    static bitis_helper::BitiaDeserializerHelper<BitisBool> deserialize(BitisDeserializer &des) {
        const auto data = des.read_bits(1);
        return bitis_helper::BitiaDeserializerHelper<BitisBool>{1, BitisBool(data == 1)};
    }
    void print(int16_t /*indent*/=0) const {
        printf("%s", (value) ? "true" : "false" );
    }
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
        const uint64_t mag = bitis_helper::magnitude(value);
        if (mag > bitis_helper::max_magnitude<T, BITS>()) { ser.set_error(BitisError::ValueOutOfRange); }
        if constexpr (std::is_signed<T>::value) {
            ser.write_bits((value < 0) ? 1 : 0, 1);
        }
        ser.write_bits(mag & bitis_helper::truncation_mask<T, BITS>(), BITS);
        return min_bits();
    }
    static bitis_helper::BitiaDeserializerHelper<IntgralWithGivenBitSize<T, BITS>>
    deserialize(BitisDeserializer &des) {
        bool is_negative = false;
        if constexpr (std::is_signed<T>::value) {
            is_negative = des.read_bits(1) != 0;
        }
        uint64_t mag = des.read_bits(BITS);
        if (mag > bitis_helper::max_magnitude<T, BITS>()) {
            des.set_error(BitisError::InvalidValue);
            mag = 0;
        }
        return bitis_helper::BitiaDeserializerHelper<IntgralWithGivenBitSize>{
            min_bits(), IntgralWithGivenBitSize(bitis_helper::from_sign_magnitude<T>(is_negative, mag))
        };
    }
    void print(int16_t /*indent*/=0) const {
        if constexpr (std::is_signed<T>::value) {
            printf("%" PRIi64 " [int_%d]", static_cast<int64_t>(value), BITS);
        }
        else {
            printf("%" PRIu64 " [uint_%d]", static_cast<uint64_t>(value), BITS);
        }
    }
    bool is_equal(const IntgralWithGivenBitSize &other) const {
        return value == other.value;
    }
    bool operator==(const IntgralWithGivenBitSize& other) const { return is_equal(other); }
    bool operator!=(const IntgralWithGivenBitSize& other) const { return !is_equal(other); }
};

template <typename T>
struct BitisFloatingPoint {
    static_assert(std::is_same<T, float>::value || std::is_same<T, double>::value,
        "float or double required for BitisFloatingPoint.");

    T value;

    BitisFloatingPoint() : value() {}
    explicit BitisFloatingPoint(T v) : value(v) { }

    static constexpr std::size_t min_bits() { return sizeof(T)*8; }

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
    void print(int16_t /*indent*/=0) const {
        printf("%f [%s]", static_cast<double>(value), (std::is_same<T, float>::value) ? "float" : "double");
    }
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

    std::size_t serialize(BitisSerializer &ser) const {
        std::size_t num_bits = 1;
        BitisBool(!is_none).serialize(ser);
        if (!is_none) {
            num_bits += value.serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<BitisOptional> deserialize(BitisDeserializer &des) {
        std::size_t num_bits = 0;
        const auto r = BitisBool::deserialize(des);
        num_bits += r.bits;
        const bool is_none = !r.data.value;
        if (!is_none) {
            auto rr = T::deserialize(des);
            num_bits += rr.bits;
            return bitis_helper::BitiaDeserializerHelper<BitisOptional>{num_bits,
                BitisOptional::create_val(std::move(rr.data))};
        }
        return bitis_helper::BitiaDeserializerHelper<BitisOptional>{num_bits,
            BitisOptional::create_none()};
    }
    void print(int16_t indent=0) const {
        if (is_none) { printf("None [Optional]"); }
        else {
            value.print(indent);
            printf("[Optional]");
        }
    }
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
        const std::size_t start_bits = ser.total_bits;
        uint64_t tval = bitis_helper::magnitude(value);
        if (tval > bitis_helper::max_magnitude<T, MAX_BITS>()) {
            // legacy behaviour: truncate (but never hang)
            ser.set_error(BitisError::ValueOutOfRange);
            tval &= bitis_helper::truncation_mask<T, MAX_BITS>();
        }
        // indicator of the sign: send one if negative (always written for signed types)
        if constexpr (std::is_signed<T>::value) {
            ser.write_bits((value < 0) ? 1 : 0, 1);
        }
        // first marker
        ser.write_bits((tval != 0) ? 1 : 0, 1);
        if (tval != 0) {
            uint8_t max_num_bits_left = MAX_BITS;
            while (true) {
                const uint8_t cur_num_bits = std::min(max_num_bits_left, BIT_PACKS);
                ser.write_bits(tval, cur_num_bits);
                tval = (cur_num_bits >= 64) ? 0 : (tval >> cur_num_bits);
                max_num_bits_left = static_cast<uint8_t>(max_num_bits_left - cur_num_bits);
                if (max_num_bits_left == 0) { break; }
                ser.write_bits((tval != 0) ? 1 : 0, 1);
                if (tval == 0) { break; }
            }
        }
        return ser.total_bits - start_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<DynInteger> deserialize(BitisDeserializer &des) {
        const std::size_t start_bits = des.consumed_bits();
        bool is_negative = false;
        if constexpr (std::is_signed<T>::value) {
            is_negative = des.read_bits(1) != 0;
        }

        uint64_t tval = 0;
        bool further_data_to_read = des.read_bits(1) != 0;
        uint8_t shift_bits = 0;
        uint8_t max_num_bits_left = MAX_BITS;
        while (further_data_to_read && !des.has_error()) {
            const uint8_t cur_num_bits = std::min(max_num_bits_left, BIT_PACKS);
            const uint64_t cdata = des.read_bits(cur_num_bits);
            tval |= cdata << shift_bits;   // shift_bits + cur_num_bits <= MAX_BITS <= 64
            shift_bits = static_cast<uint8_t>(shift_bits + cur_num_bits);
            max_num_bits_left = static_cast<uint8_t>(max_num_bits_left - cur_num_bits);

            if (max_num_bits_left > 0) { further_data_to_read = des.read_bits(1) != 0; }
            else { further_data_to_read = false; }
        }
        if (tval > bitis_helper::max_magnitude<T, MAX_BITS>()) {
            des.set_error(BitisError::InvalidValue);
            tval = 0;
        }
        return bitis_helper::BitiaDeserializerHelper<DynInteger>{des.consumed_bits() - start_bits,
            DynInteger(bitis_helper::from_sign_magnitude<T>(is_negative, tval))};
    }
    void print(int16_t /*indent*/=0) const {
        if constexpr (std::is_signed<T>::value) {
            printf("%" PRIi64 " [int_%dd%d]", static_cast<int64_t>(value), MAX_BITS, BIT_PACKS);
        }
        else {
            printf("%" PRIu64 " [uint_%dd%d]", static_cast<uint64_t>(value), MAX_BITS, BIT_PACKS);
        }
    }
    bool is_equal(const DynInteger &other) const {
        return value == other.value;
    }
    bool operator==(const DynInteger &other) const { return is_equal(other); }
    bool operator!=(const DynInteger &other) const { return !is_equal(other); }
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

    std::size_t serialize(BitisSerializer &ser) const {
        std::size_t num_bits = 0;
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            num_bits += values[i].serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<FixedArray> deserialize(BitisDeserializer &des) {
        std::size_t num_bits = 0;
        FixedArray r;

        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            auto dr = T::deserialize(des);
            r.values[i] = std::move(dr.data);
            num_bits += dr.bits;
            if (des.has_error()) { break; }
        }
        return bitis_helper::BitiaDeserializerHelper<FixedArray>{num_bits, std::move(r)};
    }
    void print(int16_t indent=0) const {
        printf("[");
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            values[i].print(indent);
            if (i != ARRAY_SIZE-1) printf(", ");
        }
        printf("]{%zu}", ARRAY_SIZE);
    }
    bool is_equal(const FixedArray &other) const {
        for (std::size_t i = 0; i < ARRAY_SIZE; i++) {
            if (values[i] != other.values[i]) { return false; }
        }
        return true;
    }
    bool operator==(const FixedArray& other) const { return is_equal(other); }
    bool operator!=(const FixedArray& other) const { return !is_equal(other); }
};

#define ENUM_INSTANCE(x) struct x { static constexpr const char* name = #x; }


// ***************************************************************
template <typename T, uint8_t DYN_BITS>
struct DynArray {
    std::vector<T> values;
    typedef T ValT;
    typedef DynInteger<uint32_t, 32, DYN_BITS> SizeT;

    DynArray() : values() { }
    explicit DynArray(const std::vector<T> &v) : values(v) { }

    static constexpr std::size_t min_bits() { return 1; }

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
        auto rv_size = SizeT::deserialize(des);
        DynArray r;
        const uint32_t count = rv_size.data.value;
        if (bitis_helper::check_count(des, count, bitis_min_bits<T>())) {
            r.values.reserve(count);
            for (uint32_t i=0; i < count; i++) {
                auto dr = T::deserialize(des);
                if (des.has_error()) { break; }
                r.values.push_back(std::move(dr.data));
            }
        }
        return bitis_helper::BitiaDeserializerHelper<DynArray>{des.consumed_bits() - start_bits, std::move(r)};
    }
    void print(int16_t indent=0) const {
        printf("[");
        for (std::size_t i=0; i < values.size(); i++) {
            values[i].print(indent);
            if (i != values.size()-1) printf(", ");
        }
        printf("]{dyn:%zu}", values.size());
    }
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
template <uint8_t DYN_BITS>
struct BinaryBase {
    typedef DynInteger<uint32_t, 32, DYN_BITS> SizeT;

    BinaryBase() : values() {}
    explicit BinaryBase(const std::vector<uint8_t> &v) : values(v) {}

    std::vector<uint8_t> *get_u8_vec() { return &this->values; }
    const std::vector<uint8_t> *get_u8_vec() const { return &this->values; }

    static constexpr std::size_t min_bits() { return 1; }

    std::size_t serialize(BitisSerializer &ser) const {
        return serialize_bytes(ser, values.data(), values.size(), false);
    }

protected:
    static std::size_t serialize_bytes(BitisSerializer &ser, const uint8_t *data, std::size_t size,
                                       const bool ascii_only) {
        std::size_t num_bits = 0;
        if (size > UINT32_MAX) { ser.set_error(BitisError::ValueOutOfRange); size = UINT32_MAX; }
        auto data_size = SizeT(static_cast<uint32_t>(size));
        num_bits += data_size.serialize(ser);

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
    static std::size_t deserialize_bytes(BitisDeserializer &des, std::vector<uint8_t> &values,
                                         const bool ascii_only, const std::size_t reserve_extra) {
        const std::size_t start_bits = des.consumed_bits();
        values.clear();
        auto rv_size = SizeT::deserialize(des);
        const uint32_t count = rv_size.data.value;
        if (bitis_helper::check_count(des, count, 8)) {
            values.reserve(count + reserve_extra);
            for (uint32_t i=0; i < count; i++) {
                const auto cdata = static_cast<uint8_t>(des.read_bits(8));
                if (ascii_only && cdata >= 0x80) { des.set_error(BitisError::InvalidValue); }
                if (des.has_error()) { values.clear(); break; }
                values.push_back(cdata);
            }
        }
        return des.consumed_bits() - start_bits;
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

    void print(int16_t /*indent*/=0) const {
        printf("[");
        for (std::size_t i=0; i < this->values.size(); i++) {
            printf((i == 0) ? "%02X" : " %02X", this->values[i]);
        }
        printf("] {binary_d%u}", DYN_BITS);
    }

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

    void print(int16_t /*indent*/=0) const {
        printf("'");
        for (std::size_t i=0; i < size(); i++) {
            printf("%c", this->values[i]);
        }
        printf("' {d%u}", DYN_BITS);
    }
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

// fp_N[min, max] (spec 2.7)
template <uint8_t BITS, int64_t MIN_IVALUE, int64_t MAX_IVALUE>
struct FixPrecisionMinMax {
    static_assert(BITS >= 2 && BITS <= 64, "FixPrecisionMinMax: BITS has to be in 2..64 (spec: 2..53).");
    static_assert(MIN_IVALUE < MAX_IVALUE, "FixPrecisionMinMax: min has to be lower than max.");

    FixPrecisionMinMax() : value(0.), state(Overflow) { }
    explicit FixPrecisionMinMax(const double &v) : value(0.), state(Ok) {
        set(v);
    }

    static constexpr std::size_t min_bits() { return BITS; }

    // stores the quantized value (or Underflow/Overflow; NaN -> Underflow)
    void set(const double &v) {
        value = 0; state = Ok;
        FixPrecisionMinMax::u64_to_val(val_to_u64(v, Ok), this->value, this->state);
        if (state != Ok) { value = 0; }
    }
    void set_underflow() { value = 0; state = Underflow; }
    void set_overflow() { value = 0; state = Overflow; }
    double get_value() const { return value; }
    FixPrecisionMinMaxEnum get_state() const { return state; }
    bool has_value() const { return state == Ok; }
    bool is_underflow() const { return state == Underflow; }
    bool is_overflow() const { return state == Overflow; }

    static uint64_t val_to_u64(double value, FixPrecisionMinMaxEnum state) {
        const uint64_t max_value = calc_bitmask(BITS);
        if (state == Underflow) { return 0; }
        if (state == Overflow) { return max_value; }

        const double min_v = static_cast<double>(MIN_IVALUE);
        const double max_v = static_cast<double>(MAX_IVALUE);
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
    static void u64_to_val(uint64_t vu, double &value, FixPrecisionMinMaxEnum &state) {
        const uint64_t max_value = calc_bitmask(BITS);

        if(vu == 0) { state = Underflow; }
        else if(vu == max_value) { state = Overflow; }
        else {
            state = Ok;
            // v = (r - 1) / S * R + min
            const double min_v = static_cast<double>(MIN_IVALUE);
            const double range = static_cast<double>(MAX_IVALUE) - min_v;
            const double steps = static_cast<double>(max_value - 2);
            const volatile double scaled = static_cast<double>(vu - 1) / steps * range;
            value = scaled + min_v;
        }
    }

    std::size_t serialize(BitisSerializer &ser) const {
        const uint64_t v = FixPrecisionMinMax::val_to_u64(value, state);
        ser.write_bits(v, BITS);
        return BITS;
    }
    static bitis_helper::BitiaDeserializerHelper<FixPrecisionMinMax> deserialize(BitisDeserializer &des) {
        FixPrecisionMinMax d;
        const uint64_t raw = des.read_bits(BITS);
        d.value = 0.;
        FixPrecisionMinMax::u64_to_val(raw, d.value, d.state);
        return bitis_helper::BitiaDeserializerHelper<FixPrecisionMinMax>{BITS, d};
    }
    void print(int16_t /*indent*/=0) const {
        if (state == Underflow) {
            printf("Underflow");
        }
        else if (state == Overflow) {
            printf("Overflow");
        }
        else {
            printf("%f", value);
        }
        printf(" [FP[%" PRIi64 ", %" PRIi64 "]]", MIN_IVALUE, MAX_IVALUE);
    }

    bool is_equal(const FixPrecisionMinMax &other) const {
        if (state == Underflow) {
            if (other.state != Underflow) { return false; }
        }
        else if (state == Overflow) {
            if (other.state != Overflow) { return false; }
        }
        else {
            if (other.state != Ok || value != other.value) { return false; }
        }
        return true;
    }
    bool operator==(const FixPrecisionMinMax &other) const { return is_equal(other); }
    bool operator!=(const FixPrecisionMinMax &other) const { return !is_equal(other); }

private:
    double value;
    FixPrecisionMinMaxEnum state;
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
    static const char *get_name(uint32_t enum_id) {
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
        auto r = DynInteger<uint32_t, 32, DYN_BITS>::deserialize(des);
        uint32_t id = r.data.value;
        if (!des.has_error() && id >= num_variants()) {
            des.set_error(BitisError::InvalidDiscriminant);
        }
        if (des.has_error()) { id = bitis_enum_helper::get_id<DEFAULT_ENUM, EnumeratedEnumCollector>(); }
        return bitis_helper::BitiaDeserializerHelper<BitisEnum>{r.bits, BitisEnum(id)};
    }

    void print(int16_t /*indent*/=0) const {
        auto v = bitis_enum_helper::get_name<EnumeratedEnumCollector>(value.value);
        // ReSharper disable once CppCStyleCast
        printf("%s (id:%" PRIu32 ") [enum_%u]", v ? v : "?", value.value, DYN_BITS);
    }

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
    template<> struct MessageT<> : MessageTE<> { };
    template<typename ... Ts>
    struct MessageT : MessageTE<typename EnumeratedList<Ts...>::type> { };

    template<typename ...> struct MessageMinBits;
    template<typename ... Ts> struct MessageMinBits<MessageT<Ts...>> {
        static constexpr std::size_t value() { return (static_cast<std::size_t>(0) + ... + bitis_min_bits<Ts>()); }
    };


    template<typename ...> struct MessageT_Impl;
    template<typename Msg_STRUCT, typename T, typename ... Ts>
    struct MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<T, Ts ...>> {
        size_t serialize(const Msg_STRUCT *d, BitisSerializer &ser) {
            auto dd = static_cast<const MessageAttribute<T>*>(d);

            std::size_t r = dd->value.serialize(ser);

            MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return r + inner.serialize(d, ser);
        }
        size_t deserialize(Msg_STRUCT *d, BitisDeserializer &des) {
            auto dd = static_cast<MessageAttribute<T>*>(d);

            bitis_helper::BitiaDeserializerHelper<typename MessageAttribute<T>::type> r =
                MessageAttribute<T>::type::deserialize(des);
            dd->value = std::move(r.data);

            MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return r.bits + inner.deserialize(d, des);
        }

        void print(const Msg_STRUCT *d, int16_t indent, const char * const *attr_names) {
            print_indent(indent);
            printf("%s: ", attr_names[MessageAttribute<T>::id::value]);

            auto dd = static_cast<const MessageAttribute<T>*>(d);
            dd->value.print(indent);
            if (indent >= 0) printf(",\n");
            else printf(", ");

            MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.print(d, indent, attr_names);
        }
    };
    template<typename Msg_STRUCT>
    struct MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<>> {
        // ReSharper disable once CppMemberFunctionMayBeStatic
        size_t serialize(const Msg_STRUCT *, BitisSerializer &) { return 0; }
        size_t deserialize(Msg_STRUCT *, BitisDeserializer &) { return 0; }
        void print(const Msg_STRUCT *, int16_t , const char * const *) { }
    };

    template<typename ...> struct MessageT_ImplStart;
    template<>
    struct MessageT_ImplStart<MessageT<>> {
        size_t serialize(const MessageT<> *, BitisSerializer &) { return 0; }
        size_t deserialize(MessageT<> *, BitisDeserializer &) { return 0; }
        void print(const MessageT<> *, int16_t, const char * const *) { }
    };
    template<typename ... Ts>
    struct MessageT_ImplStart<MessageT<Ts ...>> {
        size_t serialize(const MessageT<Ts ...> *d, BitisSerializer &ser) {
            MessageT_Impl<MessageT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            return inner.serialize(d, ser);
        }
        size_t deserialize(MessageT<Ts ...> *d, BitisDeserializer &des) {
            MessageT_Impl<MessageT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            return inner.deserialize(d, des);
        }
        void print(const MessageT<Ts ...> *d, int16_t indent, const char * const *attr_names) {
            MessageT_Impl<MessageT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            return inner.print(d, indent, attr_names);
        }
    };

    // Note: the attributes are accessed through MsgT, which has to have the same layout
    // as the generated message struct (attributes first, in declaration order).
    template<typename Msg>
    size_t msg_serialize(const Msg *d, BitisSerializer &ser) {
        typedef typename std::remove_const<Msg>::type MsgNC;
        auto *dt = reinterpret_cast<const typename MsgNC::MsgT*>(d);

        MessageT_ImplStart<typename MsgNC::MsgT> inner;
        return inner.serialize(dt, ser);
    }
    template<typename Msg>
    static bitis_helper::BitiaDeserializerHelper<Msg> msg_deserialize(BitisDeserializer &des) {
        Msg buffer;
        auto *dt = reinterpret_cast<typename Msg::MsgT*>(&buffer);

        MessageT_ImplStart<typename Msg::MsgT> inner;
        auto r = inner.deserialize(dt, des);

        return bitis_helper::BitiaDeserializerHelper<Msg>{r, std::move(buffer)};
    }
    template<typename Msg>
    void msg_print(const Msg *d, int16_t indent, const char * const *attr_names) {
        typedef typename std::remove_const<Msg>::type MsgNC;
        auto *dt = reinterpret_cast<const typename MsgNC::MsgT*>(d);
        MessageT_ImplStart<typename MsgNC::MsgT> inner;
        return inner.print(dt, indent, attr_names);
    }
    template<typename Msg>
    constexpr std::size_t msg_min_bits() {
        return MessageMinBits<typename Msg::MsgT>::value();
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
            T tmp(std::forward<Args>(args)...);   // args may refer to the stored value
            reset_impl();
            ::new (static_cast<void*>(data)) T(std::move(tmp));
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

    template<typename ...> struct OneOfT_Impl;
    template<typename OOT_STRUCT, typename T, typename ... Ts>
    struct OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<T, Ts ...>> {
        size_t serialize(const OOT_STRUCT *d, BitisSerializer &ser) {
            if (d->oo_selector.template is_enum<T>()) {
                return d->template get_oo<T>()->serialize(ser);
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.serialize(d, ser);
        }
        size_t deserialize_union(OOT_STRUCT *d, BitisDeserializer &des) {
            if (d->oo_selector.template is_enum<T>()) {
                auto r = T::OOType::deserialize(des);
                d->oo_value.set(std::move(r.data));
                return r.bits;
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.deserialize_union(d, des);
        }
        void oneof_print(const OOT_STRUCT *d, int16_t indent) {
            if (d->oo_selector.template is_enum<T>()) {
                d->template get_oo<T>()->print(indent);
                return;
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.oneof_print(d, indent);
        }
        bool is_equal(const OOT_STRUCT *self, const OOT_STRUCT *other) const {
            const bool self_sel = self->oo_selector.template is_enum<T>();
            const bool other_sel = other->oo_selector.template is_enum<T>();
            if (self_sel || other_sel) {
                if (!self_sel || !other_sel) { return false; }
                return self->template get_oo<T>()->is_equal(*other->template get_oo<T>());
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.is_equal(self, other);
        }
    };
    template<typename OOT_STRUCT>
    struct OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<>> {
        // ReSharper disable once CppMemberFunctionMayBeStatic
        size_t serialize(const OOT_STRUCT *, BitisSerializer &ser) {
            ser.set_error(BitisError::ValueOutOfRange);   // invalid selector
            return 0;
        }
        size_t deserialize_union(OOT_STRUCT *, BitisDeserializer &des) {
            des.set_error(BitisError::InvalidDiscriminant);
            return 0;
        }
        void oneof_print(const OOT_STRUCT *, int16_t ) { printf("?"); }
        bool is_equal(const OOT_STRUCT *, const OOT_STRUCT *) const  { return false; }
    };

    template<typename ...> struct OneOfT_ImplStart;
    template<typename OOT_STRUCT, typename ... Ts>
    struct OneOfT_ImplStart<OOT_STRUCT, bitis_helper::Collector<Ts ...>> {
        size_t serialize(const OOT_STRUCT *d, BitisSerializer &ser) {
            // serialize enum for selected type
            auto bits_num = d->oo_selector.serialize(ser);

            // serialize selected type
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return bits_num + inner.serialize(d, ser);
        }
        static bitis_helper::BitiaDeserializerHelper<OOT_STRUCT> deserialize(BitisDeserializer &des) {
            std::size_t bit_num = 0;

            auto d = OOT_STRUCT();
            auto r = OOT_STRUCT::T_OOEnum::deserialize(des);
            bit_num += r.bits;
            if (!des.has_error()) {
                d.oo_selector = r.data;

                OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
                bit_num += inner.deserialize_union(&d, des);
            }
            return bitis_helper::BitiaDeserializerHelper<OOT_STRUCT>{bit_num, std::move(d)};
        }
        void oneof_print(const OOT_STRUCT *d, int16_t indent) {
            d->oo_selector.print(indent);
            printf(" -> ");
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            inner.oneof_print(d, indent);
        }
        bool is_equal(const OOT_STRUCT *self, const OOT_STRUCT *other) const {
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.is_equal(self, other);
        }
    };

    template<typename OOTS>
    size_t oneof_serialize(const OOTS *d, BitisSerializer &ser) {
        typedef typename std::remove_const<OOTS>::type OOTS_NC;
        OneOfT_ImplStart<OOTS_NC, typename OOTS_NC::T_OOEnum::EnumCollector> inner;
        return inner.serialize(d, ser);
    }

    template<typename OOTS>
    BitiaDeserializerHelper<OOTS> oneof_deserialize(BitisDeserializer &des) {
        OneOfT_ImplStart<OOTS, typename OOTS::T_OOEnum::EnumCollector> inner;
        return inner.deserialize(des);
    }

    template<typename OOTS>
    void oneof_print(const OOTS *d, int16_t indent) {
        typedef typename std::remove_const<OOTS>::type OOTS_NC;
        OneOfT_ImplStart<OOTS_NC, typename OOTS_NC::T_OOEnum::EnumCollector> inner;
        inner.oneof_print(d, indent);
    }

    template<typename OOTS>
    bool oneof_is_equal(const OOTS *self, const OOTS *other) {
        typedef typename std::remove_const<OOTS>::type OOTS_NC;
        OneOfT_ImplStart<OOTS_NC, typename OOTS_NC::T_OOEnum::EnumCollector> inner;
        return inner.is_equal(self, other);
    }
    template<typename OOTS>
    constexpr std::size_t oneof_min_bits() {
        return 1 + OneOfMinBits<typename OOTS::T_OOEnum::EnumCollector>::value();
    }
}

// ***************************************************************
namespace bitis_helper {
    template<typename T, typename = void> struct HasMinBits : std::false_type { };
    template<typename T> struct HasMinBits<T, std::void_t<decltype(T::min_bits())>> : std::true_type { };
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
