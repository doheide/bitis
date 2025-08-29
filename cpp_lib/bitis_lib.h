
#include <cfloat>
#include <cinttypes>
#include <type_traits>
#include <limits>
#include <cstdint>
#include <vector>
#include <array>
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <cstring>
#include <inttypes.h>


inline void print_indent(const int16_t indent) {
    for(int i=0; i < indent; i++) { printf(" "); }
}

template<typename T>
uint64_t calc_bitmask(T bits) {
    static_assert(std::is_integral<T>::value, "calc_bitmask(): T must be an integral type");

    uint64_t val = 1;
    if (bits == 0) return 0;
    for (T i = 1; i < bits; ++i) {
        val = (val << 1) + 1;
    }
    return val;
}
static const uint8_t bit_masks[] = {0, 1, 3, 7, 15, 31, 63, 127, 255};
static const uint8_t max_bitmask = 8;

struct BitisSize {
    std::size_t bits;
    std::size_t bytes;
};


struct BitisSerializer {
    uint16_t cur_cache_u8;
    uint8_t sub_byte_counter;
    std::vector<uint8_t> data_cache;
    std::size_t final_total_bits;

    template<typename T>
    std::size_t add_data_bits(const T *value, uint8_t bits_left) {
        uint8_t bits_left_work = bits_left;
        // ReSharper disable once CppCStyleCast
        const auto data = (uint8_t*) value;

        uint8_t data_index = 0;

        while (bits_left_work > 0) {
            const uint8_t bits_to_use = std::min(bits_left_work, max_bitmask);

            const uint8_t masked_data = data[data_index] & bit_masks[bits_to_use];
            cur_cache_u8 += masked_data << sub_byte_counter;

            sub_byte_counter += bits_to_use;
            if (sub_byte_counter >= 8) {
                data_cache.push_back(static_cast<uint8_t>(cur_cache_u8 & 255));
                sub_byte_counter -= 8;
                cur_cache_u8 >>= 8;
                data_index++;
            }
            bits_left_work -= bits_to_use;
        }
        return bits_left;
    }

    template<typename T, std::size_t BITS>
    std::size_t add_data(const T *value) {
        return add_data_bits(value, BITS);
    }

    BitisSize finalize() {
        final_total_bits = data_cache.size() * 8;
        if (sub_byte_counter > 0) {
            data_cache.push_back(static_cast<uint8_t>(cur_cache_u8 & 255));
            final_total_bits += sub_byte_counter;
        }
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

struct BitisDeserializer {
    std::size_t cur_read_pos;
    uint8_t sub_byte_counter;
    std::vector<uint8_t> data_cache;

    explicit BitisDeserializer(const std::vector<uint8_t> &data)
    : cur_read_pos(0), sub_byte_counter(0), data_cache(data) { }

    bitis_helper::BitiaDeserializerUint8Helper decode_data_uint8(const uint8_t total_bits) {
        // if (cur_read_pos >= total_bits) {
        //     return bitis_helper::BitiaDeserializerUint8Helper{.bits = 0, .data = 0};
        // }
        uint16_t cur_u16 = data_cache[cur_read_pos];
        if (cur_read_pos+1 < data_cache.size()) {
            // todo what is this for?
            cur_u16 += data_cache[cur_read_pos+1] << 8;
        }
        const auto bits_to_use = std::min(total_bits, max_bitmask);
        const auto d = (uint8_t) ((cur_u16 >> sub_byte_counter) & bit_masks[bits_to_use]);
        sub_byte_counter += bits_to_use;
        if (sub_byte_counter >= 8) {
            cur_read_pos ++;
            sub_byte_counter &= 7;
        }
        return bitis_helper::BitiaDeserializerUint8Helper{.bits = bits_to_use, .data = d};
    }
    template<typename T>
    // bitis_helper::BitiaDeserializerHelper<T> decode_data(T & data, const uint8_t total_bits) {
    std::size_t  decode_data(T & data, const uint8_t total_bits) {
        uint8_t bits_left = total_bits;
        // T data;
        // ReSharper disable once CppCStyleCast
        auto *data_ptr = (uint8_t *) &data;
        uint8_t idx = 0;

        while (bits_left > 0) {
            const auto rd = decode_data_uint8(bits_left);
            data_ptr[idx] = rd.data;
            bits_left -= rd.bits;
            idx++;
        }
        // return bitis_helper::BitiaDeserializerHelper<T>{.bits = total_bits, .data = data};
        return total_bits;
    }
};


// ***************************************************************
struct BitisBool {
    bool value;

    BitisBool() : value(false) {}
    BitisBool(const bool value) : value(value) {}

    // ReSharper disable once CppDFAConstantFunctionResult
    std::size_t serialize(BitisSerializer &ser) {
        const uint8_t data = value ? 1 : 0;
        return ser.add_data<uint8_t, 1>(&data);
    }
    static bitis_helper::BitiaDeserializerHelper<BitisBool> deserialize(BitisDeserializer &des) {
        uint8_t data = 0;
        des.decode_data<uint8_t>(data, 1);
        return bitis_helper::BitiaDeserializerHelper<BitisBool>{.bits = 1, .data = BitisBool(data==1)
        };
    }
    void print(int16_t indent=0) const {
        printf("%s", (value) ? "true" : "false" );
    }
    bool is_equal(const BitisBool &other) const {
        return value == other.value;
    }
    bool operator==(const BitisBool &other) const { return is_equal(other); }
    bool operator!=(const BitisBool &other) const { return !is_equal(other); }
};

// ***************************************************************
template <typename T, uint8_t BITS>
struct IntgralWithGivenBitSize {
    static_assert(std::is_integral<T>::value, "Integral type required for IntgralWithGivenBitSize.");
    static_assert(std::numeric_limits<T>::digits >= BITS, "BITS have to be lower or equal than the number of bits of the type.");

    T value;

    IntgralWithGivenBitSize() : value() {}
    explicit IntgralWithGivenBitSize(T value) : value(value) {}
    // IntgralWithGivenBitSize(T &value) : value(value) {}

    std::size_t serialize(BitisSerializer &ser) {
        uint8_t num_bits = 0;
        T tval = value;

        if (std::is_signed<T>::value) {
            uint8_t tdata;

            if (tval < 0) {
                tdata = 1;
                tval = -tval;
            }
            else tdata = 0;

            ser.add_data<uint8_t, 1>(&tdata);
            num_bits += 1;
        }
        return ser.add_data<T, BITS>(&tval) + num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<IntgralWithGivenBitSize<T, BITS>>
    deserialize(BitisDeserializer &des) {
        std::size_t num_bits = 0;
        bool is_negative = false;
        if (std::is_signed<T>::value) {
            const auto r = BitisBool::deserialize(des);
            num_bits += r.bits;
            is_negative = r.data.value;
        }
        T v(0);
        auto dr = des.decode_data(v, BITS);
        if (std::is_signed<T>::value) {
            if (is_negative) {
                v = -v;
            }
        }
        return  bitis_helper::BitiaDeserializerHelper<IntgralWithGivenBitSize>{
            .bits = num_bits+dr, .data = IntgralWithGivenBitSize(v)
        };
    }
    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("%" PRIi64 " [%sint_%d]", (int64_t)value, (std::is_signed<T>::value) ? "" : "u", BITS);
    }
    bool is_equal(const IntgralWithGivenBitSize &other) const {
        return value == other.value;
    }
    bool operator==(const IntgralWithGivenBitSize& other) const { return is_equal(other); }
    bool operator!=(const IntgralWithGivenBitSize& other) const { return !is_equal(other); }
};

template <typename T>
struct BitisFloatingPoint {
    static_assert(std::is_floating_point<T>::value, "Floating point type required for BitisFloatingPoint.");

    T value;

    BitisFloatingPoint() : value() {}
    explicit BitisFloatingPoint(T v) : value(v) { }

    std::size_t serialize(BitisSerializer &ser) {
        return ser.add_data<T, sizeof(T)*8>(&value) ;
    }
    static bitis_helper::BitiaDeserializerHelper<BitisFloatingPoint> deserialize(
        BitisDeserializer &des) {
        T val(0.0);
        auto r = des.decode_data(val, sizeof(T)*8);
        return bitis_helper::BitiaDeserializerHelper<BitisFloatingPoint>{
        .bits = sizeof(T)*8, .data = BitisFloatingPoint(val)};
    }
    void print(int16_t indent=0) const {
        // ReSharper disable once CppCStyleCast
        printf("%f [%s]", value, (std::numeric_limits<T>::digits == FLT_MANT_DIG) ? "float" : "double");
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
    BitisOptional(T val) : value(val), is_none(false) {}
    explicit BitisOptional(T value, bool _is_none) : value(value), is_none(_is_none) {}
    static BitisOptional create_none() { return BitisOptional(T(), true); }
    static BitisOptional create_val(T v) { return BitisOptional(v, false); }

    std::size_t serialize(BitisSerializer &ser) {
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
            return bitis_helper::BitiaDeserializerHelper<BitisOptional>{.bits = num_bits,
                .data = BitisOptional::create_val(rr.data)};
        }
        return bitis_helper::BitiaDeserializerHelper<BitisOptional>{.bits = num_bits,
            .data = BitisOptional::create_none()};
    }
    void print(int16_t indent=0) {
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
template <typename T, uint8_t MAX_BITS, uint8_t BIT_PACKS>
struct DynInteger {
    T value;

    DynInteger() : value() {}
    explicit DynInteger(T value) : value(value) {}

    std::size_t serialize(BitisSerializer &ser) {
        int num_bits = 1;
        uint8_t tdata;
        T tvalue = value;
        // indicator of the sign: send one if negative
        if (std::is_signed<T>::value) {
            if (value < 0) {
                tdata = 1;
                tvalue = -tvalue;
            }
            else tdata = 0;

            ser.add_data<uint8_t, 1>(&tdata);
            num_bits += 1;
        }
        // tvalue is a positive value here

        // first marker
        {   auto b = BitisBool(tvalue>0);
            b.serialize(ser);
        }
        //
        uint64_t tval = tvalue; // should be always a positive value
        uint8_t max_num_bits_left = MAX_BITS;
        while (tval > 0) {
            const auto cur_num_bits = std::min(max_num_bits_left, BIT_PACKS);
            ser.add_data_bits(&tval, cur_num_bits);

            tval >>= cur_num_bits;
            num_bits += cur_num_bits;
            max_num_bits_left -= cur_num_bits;

            if (max_num_bits_left > 0) {
                auto b = BitisBool(tval>0);
                b.serialize(ser);
                num_bits += 1;
            }
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<DynInteger> deserialize(BitisDeserializer &des) {
        bool is_negative = false;
        uint8_t bits_num = 1;
        uint64_t tval = 0;

        if (std::is_signed<T>::value) {
            auto rv = BitisBool::deserialize(des);
            bits_num += rv.bits;
            is_negative = rv.data.value;
        }

        auto rv = BitisBool::deserialize(des);
        bool further_data_to_read = rv.data.value;
        uint8_t shift_bits = 0;
        uint8_t max_num_bits_left = MAX_BITS;
        while (further_data_to_read) {
            const auto cur_num_bits = std::min(max_num_bits_left, BIT_PACKS);

            uint64_t cdata = 0;
            const auto r = des.decode_data(cdata, cur_num_bits);

            bits_num += r;
            tval += cdata << shift_bits;
            shift_bits += cur_num_bits;
            max_num_bits_left -= cur_num_bits;

            if (max_num_bits_left > 0) {
                rv = BitisBool::deserialize(des);
                further_data_to_read = rv.data.value;
                bits_num += 1;
            }
            else { further_data_to_read = false; }
        }
        if (is_negative) {
            tval = -tval;
        }
        // ReSharper disable once CppCStyleCast
        return bitis_helper::BitiaDeserializerHelper<DynInteger>{.bits=bits_num, .data=DynInteger((T)tval)};
    }
    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("%" PRIi64 " [%sint_%dd%d]", (int64_t)value, (std::is_signed<T>::value) ? "" : "u", MAX_BITS, BIT_PACKS);
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

    std::size_t serialize(BitisSerializer &ser) {
        std::size_t num_bits = 0;
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            num_bits += values[i].serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<FixedArray> deserialize(BitisDeserializer &des) {
        std::size_t num_bits = 0;
        //T values[ARRAY_SIZE];
        std::array<T, ARRAY_SIZE> values;

        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            auto dr = T::deserialize(des);
            values[i] = dr.data;
            num_bits += dr.bits;
        }
        return bitis_helper::BitiaDeserializerHelper<FixedArray>{.bits = num_bits, .data = FixedArray(values)};
    }
    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("[");
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            values[i].print(indent);
            if (i != ARRAY_SIZE-1) printf(", ");
        }
        printf("]{%zu}", ARRAY_SIZE);
    }
    bool is_equal(const FixedArray &other) const {
        for (int i = 0; i < ARRAY_SIZE; i++) {
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

    DynArray() : values() { }
    explicit DynArray(const std::vector<T> &v) : values(v) { }

    std::size_t serialize(BitisSerializer &ser) {
        std::size_t num_bits = 0;
        auto data_size = DynInteger<uint32_t, 32, DYN_BITS>(values.size());
        num_bits += data_size.serialize(ser);
        for (std::size_t i=0; i < values.size(); i++) {
            // num_bits += ser.add_data(values[i].serialize(ser));
            num_bits += values[i].serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<DynArray> deserialize(BitisDeserializer &des) {
        std::size_t num_bits = 0;
        // auto tbits = sizeof(T)*8;
        auto rv_size = DynInteger<uint32_t, 32, DYN_BITS>::deserialize(des);
        num_bits += rv_size.bits;

        auto tvalues = std::vector<T>(rv_size.data.value);
        for (std::size_t i=0; i < rv_size.data.value; i++) {
            auto dr = T::deserialize(des);
            tvalues[i] = dr.data;
            num_bits += dr.bits;
            // T cdata;
            // auto bits = des.decode_data(cdata, tbits);
            // tvalues[i] = cdata;
            // num_bits += bits;
        }
        return bitis_helper::BitiaDeserializerHelper<DynArray>{.bits=num_bits, .data=DynArray(tvalues)};
    }
    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("[");
        for (std::size_t i=0; i < values.size(); i++) {
            values[i].print(indent);
            if (i != values.size()-1) printf(", ");
        }
        printf("]{dyn:%zu}", values.size());
    }
    bool is_equal(const DynArray &other) const {
        if (values.size() != other.values.size()) { return false; }
        for (int i = 0; i < values.size(); i++) {
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
    BinaryBase() : values() {}
    explicit BinaryBase(const std::vector<uint8_t> &v) : values(v) {}

    std::vector<uint8_t> *get_u8_vec() { return &this->values; }

    std::size_t serialize(BitisSerializer &ser) {
        std::size_t num_bits = 0;
        auto data_size = DynInteger<uint32_t, 32, DYN_BITS>(values.size());
        num_bits += data_size.serialize(ser);

        for (std::size_t i=0; i < values.size(); i++) {
            auto cv = values[i];
            num_bits += ser.add_data<uint8_t, 8>(&cv);
        }
        return num_bits;
    }

protected:
    static bitis_helper::BitiaDeserializerHelper<BinaryBase> deserialize_base(BitisDeserializer &des) {
        std::size_t num_bits = 0;
        auto tbits = 8;
        auto rv_size = DynInteger<uint32_t, 32, DYN_BITS>::deserialize(des);
        num_bits += rv_size.bits;

        auto tvalues = std::vector<uint8_t>(rv_size.data.value);
        for (std::size_t i=0; i < rv_size.data.value; i++) {
            uint8_t cdata = 0;
            auto bits = des.decode_data(cdata, tbits);
            tvalues[i] = cdata;
            num_bits += bits;
        }
        return bitis_helper::BitiaDeserializerHelper<BinaryBase>{.bits=num_bits, .data=BinaryBase(tvalues)};
    }

    std::vector<uint8_t> values;
};

// ***************************************************************
template <uint8_t DYN_BITS>
struct Binary : BinaryBase<DYN_BITS>{

    Binary() : BinaryBase<DYN_BITS>() {}
    explicit Binary(const std::vector<uint8_t> &v) : BinaryBase<DYN_BITS>() { }

    void set(const std::vector<uint8_t> &v) {
        this->values = v;
    }

    static bitis_helper::BitiaDeserializerHelper<Binary> deserialize(BitisDeserializer &des) {
        auto v = BinaryBase<DYN_BITS>::deserialize_base(des);
        return bitis_helper::BitiaDeserializerHelper<Binary>{.bits=v.bits, .data=Binary(*v.data.get_u8_vec())};
    }

    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("'");
        for (std::size_t i=0; i < this->value.values.size(); i++) {
            printf("%c", this->value.values[i]);
        }
        printf("' {d%u}", DYN_BITS);
    }

    bool is_equal(const Binary &other) const {
        return this->value == other.value;
    }
    bool operator==(const Binary& other) const { return is_equal(other); }
    bool operator!=(const Binary& other) const { return !is_equal(other); }
};

template <uint8_t DYN_BITS>
struct BitisAString : BinaryBase<DYN_BITS>{

    BitisAString() : BinaryBase<DYN_BITS>() {}
    explicit BitisAString(const char* s) { set(s); }
    explicit BitisAString(const std::vector<uint8_t> &v) : BinaryBase<DYN_BITS>(v) { }

    void set(const char* s) {
        this->values.clear();
        this->values.reserve(strlen(s));
        for (int i = 0; s[i] != '\0'; i++) {
            this->values.push_back(s[i]);
        }
    }
    const char *get() {
        // ReSharper disable once CppCStyleCast
        return (const char *) this->values.data();
    }

    static bitis_helper::BitiaDeserializerHelper<BitisAString> deserialize(BitisDeserializer &des) {
        auto v = BinaryBase<DYN_BITS>::deserialize_base(des);
        return bitis_helper::BitiaDeserializerHelper<BitisAString>{.bits=v.bits, .data=BitisAString(*v.data.get_u8_vec())};
    }

    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("'");
        for (std::size_t i=0; i < this->values.size(); i++) {
            printf("%c", this->values[i]);
        }
        printf("' {d%u}", DYN_BITS);
    }
    bool is_equal(const BitisAString &other) const {
        return this->values == other.values;
    }
    bool operator==(const BitisAString& other) const { return is_equal(other); }
    bool operator!=(const BitisAString& other) const { return !is_equal(other); }
};

// ***************************************************************
enum FixPrecisionMinMaxEnum {Ok, Underflow, Overflow};

template <uint8_t BITS, int64_t MIN_IVALUE, int64_t MAX_IVALUE>
struct FixPrecisionMinMax {
    static_assert(BITS > 2);

    FixPrecisionMinMax() : value(0.), state(Overflow) { }
    explicit FixPrecisionMinMax(const double &v) { // NOLINT(*-pro-type-member-init)
        set(v);
    }

    void set(const double &v) {
        value = 0; state = Ok;

        if (v < MIN_IVALUE) { state = Underflow; }
        else if (v > MAX_IVALUE) { state = Overflow; }
        else { value = v; }
        FixPrecisionMinMax::u64_to_val(val_to_u64(value, state), this->value, this->state);
    }
    double get_value() const { return value; }
    FixPrecisionMinMaxEnum get_state() const { return state; }

    static uint64_t val_to_u64(double value, FixPrecisionMinMaxEnum state) {
        const auto max_value = calc_bitmask(BITS);
        uint64_t v;
        if (state == Underflow) { v = 0; }
        else if (state == Overflow) { v = max_value; }
        else {
            auto of = (value - ((double)MIN_IVALUE));
            auto diff = (double)(MAX_IVALUE - MIN_IVALUE);
            auto mv = (double)max_value - 2.;
            v = lround(of / diff * mv) + 1;

            // v = (uint64_t) ((value - ((double)MIN_IVALUE))
            //     / ((double)(MAX_IVALUE - MIN_IVALUE))
            //     * (double)(max_value - 2)) + 1;
        }
        return v;
    }
    static void u64_to_val(uint64_t vu, double &value, FixPrecisionMinMaxEnum &state) {
        const auto max_value = calc_bitmask(BITS);

        if(vu == 0) { state = Underflow; }
        else if(vu == max_value) { state = Overflow; }
        else {
            state = Ok;
            value = (((double)vu-1.) / ((double)(max_value-2)) *
                ((double)(MAX_IVALUE-MIN_IVALUE))) + (double)MIN_IVALUE;
        }
    }

    std::size_t serialize(BitisSerializer &ser) {
        std::size_t num_bits = 0;
        // auto max_value = calc_bitmask(BITS);

        uint64_t v = FixPrecisionMinMax::val_to_u64(value, state);
/*        if (state == Underflow) { v = 0; }
        else if (state == Overflow) { v = max_value; }
        else {
            v = lround((value - ((double)MIN_IVALUE))
                / ((double)(MAX_IVALUE - MIN_IVALUE))
                * (double)(max_value - 2)) + 1;
        }*/
        auto t = IntgralWithGivenBitSize<uint64_t, BITS>{v};
        num_bits += t.serialize(ser);
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<FixPrecisionMinMax> deserialize(BitisDeserializer &des) {
        // auto max_value = calc_bitmask(BITS);

        auto d = FixPrecisionMinMax(0.0);
        auto data_u64 = IntgralWithGivenBitSize<uint64_t, BITS>::deserialize(des);
        // printf("data_u64: %" PRIu64, data_u64.data.value);
        FixPrecisionMinMax::u64_to_val(data_u64.data.value, d.value, d.state);

/*        if(data_u64.data.value == 0) {
            d.state = Underflow;
        } else if(data_u64.data.value == max_value) {
            d.state = Overflow;
        } else {
            d.state = Ok;
            d.value = (((double)data_u64.data.value-1.) / (static_cast<double>(max_value-2)) *
                ((double)(MAX_IVALUE-MIN_IVALUE))) + (double)MIN_IVALUE;
        }*/

        return bitis_helper::BitiaDeserializerHelper<FixPrecisionMinMax>{.bits = BITS, .data = d };
    }
    void print(int16_t indent=0) {
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
            if (value != other.value) { return false; }
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
        static size_t get_id() {
            return EnumID_Impl<SEL_ENUM, bitis_helper::Collector<Ts ...>>::get_id();
        }
    };
    template<typename T, typename ... Ts>
    struct EnumID_Impl<typename T::type, bitis_helper::Collector<T, Ts ...>> {
        static size_t get_id() { return T::id::value; }
    };
    template<typename SEL_ENUM>
    struct EnumID_Impl<SEL_ENUM, bitis_helper::Collector<>> {
        // ReSharper disable once CppStaticAssertFailure
        static_assert(false, "Enum unknown");
        static size_t get_id() { return 0; }
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
        static const char *get_name(uint32_t enum_id) { return nullptr; }
    };

    template<typename ENUM_TO_FIND, typename ENUMERATED_ENUM_COL>
    uint32_t get_id() {
        return EnumID_Impl<ENUM_TO_FIND, ENUMERATED_ENUM_COL>::get_id();
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

    static_assert(bitis_helper::ContainsType<DEFAULT_ENUM, EnumCollector>::value);

    explicit BitisEnum() : value(bitis_enum_helper::get_id<DEFAULT_ENUM, EnumeratedEnumCollector>()) {}

private:
    explicit BitisEnum(uint32_t enum_id) : value(enum_id) {}

public:
    template<typename SET_ENUM>
    static BitisEnum create_enum() {
        auto id_in = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
        return BitisEnum(id_in);
    }

    template<typename SET_ENUM>
    void set_enum() {
        static_assert(bitis_helper::ContainsType<SET_ENUM, EnumCollector>::value);
        this->value.value = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
    }
    template<typename SET_ENUM>
    bool is_enum() const {
        static_assert(bitis_helper::ContainsType<SET_ENUM, EnumCollector>::value);
        auto id_in = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
        return this->value.value == id_in;
    }

    std::size_t serialize(BitisSerializer &ser) {
        return value.serialize(ser);
    }
    static bitis_helper::BitiaDeserializerHelper<BitisEnum> deserialize(BitisDeserializer &des) {
        auto r = DynInteger<uint32_t, 32, DYN_BITS>::deserialize(des);
        return bitis_helper::BitiaDeserializerHelper<BitisEnum>{.bits=r.bits, .data=BitisEnum(r.data.value)};
    }

    void print(int16_t indent=0) {
        auto v = bitis_enum_helper::get_name<EnumeratedEnumCollector>(value.value);
        // ReSharper disable once CppCStyleCast
        printf("%s (id:%" PRIu32 ") [enum_%u]", v, value.value, DYN_BITS);
    }

    bool is_equal(const BitisEnum &other) const {
        return this->value == other.value;
    }
    bool operator==(const BitisEnum &other) const { return is_equal(other); }
    bool operator!=(const BitisEnum &other) const { return !is_equal(other); }

    // BitisEnum &operator=(const BitisEnum &other) {
    //     this->value = other.value;
    //     return *this;
    // }
    // BitisEnum &operator=(const DynInteger<uint32_t, DYN_BITS> &other) {
    //     this->value = other.value;
    //     return *this;
    // }
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


    template<typename ...> struct MessageT_Impl;
    template<typename Msg_STRUCT, typename T, typename ... Ts>
    struct MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<T, Ts ...>> {
        size_t serialize(Msg_STRUCT *d, BitisSerializer &ser) {
            auto dd = static_cast<MessageAttribute<T>*>(d);

            std::size_t r = dd->value.serialize(ser);

            MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return r + inner.serialize(d, ser);
        }
        size_t deserialize(Msg_STRUCT *d, BitisDeserializer &des) {
            auto dd = static_cast<MessageAttribute<T>*>(d);

            // auto dd = reinterpret_cast<MessageAttribute<T>*>(d);
            bitis_helper::BitiaDeserializerHelper<typename MessageAttribute<T>::type> r =
                MessageAttribute<T>::type::deserialize(des);
            dd->value = r.data;

            MessageT_Impl<Msg_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return r.bits + inner.deserialize(d, des);
        }

        void print(Msg_STRUCT *d, int16_t indent, const char **attr_names) {
            print_indent(indent);
            printf("%s: ", attr_names[MessageAttribute<T>::id::value]);

            auto dd = static_cast<MessageAttribute<T>*>(d);
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
        size_t serialize(Msg_STRUCT *d, BitisSerializer &ser) { return 0; }
        size_t deserialize(Msg_STRUCT *d, BitisDeserializer &) { return 0; }
        void print(Msg_STRUCT *, int16_t , const char **) { }
    };

    template<typename ...> struct MessageT_ImplStart;
    template<>
    struct MessageT_ImplStart<MessageT<>> {
        size_t serialize(MessageT<> *d, BitisSerializer &ser) { return 0; }
        size_t deserialize(MessageT<> *d, BitisDeserializer &des) { return 0; }
        void print(MessageT<> *d, int16_t indent, const char **attr_names) { }
    };
    template<typename ... Ts>
    struct MessageT_ImplStart<MessageT<Ts ...>> {
        size_t serialize(MessageT<Ts ...> *d, BitisSerializer &ser) {
            MessageT_Impl<MessageT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            return inner.serialize(d, ser);
        }
        size_t deserialize(MessageT<Ts ...> *d, BitisDeserializer &des) {
            MessageT_Impl<MessageT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            return inner.deserialize(d, des);
        }
        void print(MessageT<Ts ...> *d, int16_t indent, const char **attr_names) {
            MessageT_Impl<MessageT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            return inner.print(d, indent, attr_names);
        }
    };

    template<typename Msg>
    size_t msg_serialize(Msg *d, BitisSerializer &ser) {
        // ReSharper disable once CppCStyleCast
        auto *dt = (typename Msg::MsgT*) d;

        MessageT_ImplStart<typename Msg::MsgT> inner;
        return inner.serialize(dt, ser);
    }
    template<typename Msg>
    static bitis_helper::BitiaDeserializerHelper<Msg> msg_deserialize(BitisDeserializer &des) {
        Msg buffer;
        // ReSharper disable once CppCStyleCast
        auto *dt = (typename Msg::MsgT*) &buffer;

        MessageT_ImplStart<typename Msg::MsgT> inner;
        auto r = inner.deserialize(dt, des);

        return bitis_helper::BitiaDeserializerHelper<Msg>(
            // ReSharper disable once CppCStyleCast
            bitis_helper::BitiaDeserializerHelper<Msg>{.bits=r, .data = *((Msg*)dt)});
    }
    template<typename Msg>
    void msg_print(Msg *d, int16_t indent, const char **attr_names) {
        // ReSharper disable once CppCStyleCast
        auto *dt = (typename Msg::MsgT*) d;
        MessageT_ImplStart<typename Msg::MsgT> inner;
        return inner.print(dt, indent, attr_names);
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

    template<typename ...> struct UnionT;
    template<typename ... Ts>
    struct UnionT {
    private:
        void *data[MaxSizeof<Ts ...>::value] = {};
    public:
        template<typename T>
        void set(T v) {
            static_assert(ContainsType<T, bitis_helper::Collector<Ts...>>::value);
            *((T*)data) = v;
        }
        template<typename T>
        T *get() const {
            static_assert(ContainsType<T, bitis_helper::Collector<Ts...>>::value);
            return (T*)data;
        }
    };


    template<typename ...> struct OneOfT_Impl;
    template<typename OOT_STRUCT, typename T, typename ... Ts>
    struct OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<T, Ts ...>> {
        size_t serialize(OOT_STRUCT *d, BitisSerializer &ser) {
            // ReSharper disable once CppCStyleCast
            auto data_pointer = d->template get_oo<T>();
            if (data_pointer) {
                // ReSharper disable once CppCStyleCast
                const std::size_t r = data_pointer->serialize(ser);
                //printf("inner bits: %lu\n", r);
                return r;
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.serialize(d, ser);
        }
        size_t deserialize_union(OOT_STRUCT *d, BitisDeserializer &des) {
            auto data_pointer = d->template get_oo<typename remove_const<T>::type>();
            if (data_pointer) {
                auto r = T::OOType::deserialize(des);
                d->oo_value.set(r.data);
                return r.bits;
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.deserialize_union(d, des);
        }
        void oneof_print(OOT_STRUCT *d, int16_t indent) {
            auto data_pointer = d->template get_oo<typename remove_const<T>::type>();
            if (data_pointer) {
                data_pointer->print(indent);
                return;
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.oneof_print(d, indent);
        }
        bool is_equal(const OOT_STRUCT *self, const OOT_STRUCT *other) const {
            auto data_pointer_self = self->template get_oo<typename remove_const<T>::type>();
            auto data_pointer_other = other->template get_oo<typename remove_const<T>::type>();
            if (data_pointer_self || data_pointer_other) {
                if (data_pointer_self==nullptr || data_pointer_other==nullptr) {
                    return false; }
                return data_pointer_self->is_equal(*data_pointer_other);
            }
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return inner.is_equal(self, other);
        }
    };
    template<typename OOT_STRUCT>
    struct OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<>> {
        // ReSharper disable once CppMemberFunctionMayBeStatic
        size_t serialize(OOT_STRUCT *, BitisSerializer &) { return 0; }
        size_t deserialize_union(OOT_STRUCT *, BitisDeserializer &) { return 0; }
        void oneof_print(OOT_STRUCT *, int16_t ) { }
        bool is_equal(const OOT_STRUCT *self, const OOT_STRUCT *other) const  { return false; }
    };

    template<typename ...> struct OneOfT_ImplStart;
    template<typename OOT_STRUCT, typename ... Ts>
    struct OneOfT_ImplStart<OOT_STRUCT, bitis_helper::Collector<Ts ...>> {
        size_t serialize(OOT_STRUCT *d, BitisSerializer &ser) {
            // serialize enum for selected type
            auto bits_num = d->oo_selector.serialize(ser);
            //printf("oo_selector bits_num: %lu\n", bits_num);

            // serialize selected type
            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            return bits_num + inner.serialize(d, ser);
        }
        static bitis_helper::BitiaDeserializerHelper<OOT_STRUCT> deserialize(BitisDeserializer &des) {
            std::size_t bit_num = 0;

            auto d = OOT_STRUCT();
            auto r = OOT_STRUCT::T_OOEnum::deserialize(des);
            bit_num += r.bits;
            d.oo_selector = r.data;

            OneOfT_Impl<OOT_STRUCT, bitis_helper::Collector<Ts ...>> inner;
            bit_num += inner.deserialize_union(&d, des);
            return bitis_helper::BitiaDeserializerHelper<OOT_STRUCT>{.bits=bit_num, .data=d};
        }
        void oneof_print(OOT_STRUCT *d, int16_t indent) {
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
    size_t oneof_serialize(OOTS *d, BitisSerializer &ser) {
        OneOfT_ImplStart<OOTS, typename OOTS::T_OOEnum::EnumCollector> inner;
        return inner.serialize(d, ser);
    }

    template<typename OOTS>
    BitiaDeserializerHelper<OOTS> oneof_deserialize(BitisDeserializer &des) {
        OneOfT_ImplStart<OOTS, typename OOTS::T_OOEnum::EnumCollector> inner;
        return inner.deserialize(des);
    }

    template<typename OOTS>
    void oneof_print(OOTS *d, int16_t indent) {
        // ReSharper disable once CppCStyleCast
        OneOfT_ImplStart<OOTS, typename OOTS::T_OOEnum::EnumCollector> inner;
        inner.oneof_print(d, indent);
    }

    template<typename OOTS>
    bool oneof_is_equal(const OOTS *self, const OOTS *other) {
        // OneOfT_ImplStart<OOTS, typename OOTS::OOEnum::EnumCollector> inner;
        // return inner.serialize(d, ser);
        OneOfT_ImplStart<typename remove_const<OOTS>::type,typename OOTS::T_OOEnum::EnumCollector> inner;
        return inner.is_equal(self, other);
    }
}


template<typename T>
std::vector<uint8_t> serialize(T & msg) {
    auto ser = BitisSerializer();
    // auto r = serialize(a, ser);
    msg.serialize(ser);
    auto r = ser.finalize();
    return ser.data_cache;
}

template<typename T>
T deserialize(const std::vector<uint8_t>& data) {
    auto des = BitisDeserializer(data);
    auto dd = T::deserialize(des);
    return dd.data;
}

