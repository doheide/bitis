//
// Created by dheide on 26.03.25.
//

#pragma once

#include <cfloat>
#include "inttypes.h"
#include <type_traits>
#include <limits>
#include <cstdint>
#include <vector>
#include <array>
#include <algorithm>
#include <cmath>
#include <bits/locale_facets_nonio.h>


inline void print_indent(const int16_t indent) {
    for(int i=0; i < indent; i++) { printf(" "); }
}

template<typename T>
T calc_bitmask(T bits) {
    static_assert(std::is_integral<T>::value, "calc_bitmask(): T must be an integral type");

    T val = 1;
    if (bits == 0) return 0;
    for (T i = 1; i < bits; i++) {
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

    template<typename T, std::size_t BITS>
    std::size_t add_data(const T *value) {

        // ReSharper disable once CppCStyleCast
        const auto data = (uint8_t*) value;

        uint8_t bits_left = BITS;
        uint8_t data_index = 0;

        while (bits_left > 0) {
            const uint8_t bits_to_use = std::min(bits_left, max_bitmask);

            const uint8_t masked_data = data[data_index] & bit_masks[bits_to_use];
            cur_cache_u8 += masked_data << sub_byte_counter;

            sub_byte_counter += bits_to_use;
            if (sub_byte_counter >= 8) {
                data_cache.push_back(static_cast<uint8_t>(cur_cache_u8 & 255));
                sub_byte_counter -= 8;
                cur_cache_u8 >>= 8;
                data_index++;
            }
            bits_left -= bits_to_use;
        }
        return BITS;
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
    bitis_helper::BitiaDeserializerHelper<T> decode_data(const uint8_t total_bits) {
        uint8_t bits_left = total_bits;
        T data(0);
        // ReSharper disable once CppCStyleCast
        auto *data_ptr = (uint8_t *) &data;
        uint8_t idx = 0;

        while (bits_left > 0) {
            const auto rd = decode_data_uint8(bits_left);
            data_ptr[idx] = rd.data;
            bits_left -= rd.bits;
            idx++;
        }
        return bitis_helper::BitiaDeserializerHelper<T>{.bits = total_bits, .data = data};
    }
};


// ***************************************************************
struct BitisBool {
    bool value;

    BitisBool() : value(false) {}
    explicit BitisBool(const bool value) : value(value) {}

    // ReSharper disable once CppDFAConstantFunctionResult
    std::size_t serialize(BitisSerializer &ser) const {
        const uint8_t data = value ? 1 : 0;
        return ser.add_data<uint8_t, 1>(&data);
    }
    static bitis_helper::BitiaDeserializerHelper<BitisBool> deserialize(BitisDeserializer &des) {
        const auto dr = des.decode_data<uint8_t>(1);
        return bitis_helper::BitiaDeserializerHelper<BitisBool>{
            .bits = 1, .data = BitisBool(dr.data==1)
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

    std::size_t serialize(BitisSerializer &ser) const {
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
            num_bits += BITS;
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
        auto dr = des.decode_data<T>(BITS);
        if (std::is_signed<T>::value) {
            if (is_negative) {
                dr.data = -dr.data;
            }
        }
        return  bitis_helper::BitiaDeserializerHelper<IntgralWithGivenBitSize>{
            .bits = num_bits, .data = IntgralWithGivenBitSize(dr.data)
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

    std::size_t serialize(BitisSerializer &ser) const {
        return ser.add_data<T, sizeof(T)*8>(&value) ;
    }
    static bitis_helper::BitiaDeserializerHelper<BitisFloatingPoint> deserialize(
        BitisDeserializer &des) {
        auto r = des.decode_data<T>(sizeof(T)*8);
        return bitis_helper::BitiaDeserializerHelper<BitisFloatingPoint>{
        .bits = sizeof(T)*8, .data = BitisFloatingPoint(r.data)};
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
template <typename T, uint8_t DYN_BITS>
struct DynInteger {
    T value;

    DynInteger() : value() {}
    explicit DynInteger(T value) : value(value) {}

    std::size_t serialize(BitisSerializer &ser) const {
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
        {   const auto b = BitisBool(tvalue>0);
            b.serialize(ser);
        }
        //
        uint64_t tval = tvalue; // should be always a positive value
        while (tval > 0) {
            ser.add_data<uint64_t, DYN_BITS>(&tval);

            tval >>= DYN_BITS;
            num_bits += DYN_BITS + 1;

            const auto b = BitisBool(tval>0);
            b.serialize(ser);
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
        while (further_data_to_read) {
            const auto r = des.decode_data<uint64_t>(DYN_BITS);

            bits_num += r.bits + 1;
            tval += r.data << shift_bits;
            shift_bits += DYN_BITS;

            rv = BitisBool::deserialize(des);
            further_data_to_read = rv.data.value;
        }
        if (is_negative) {
            tval = -tval;
        }
        return bitis_helper::BitiaDeserializerHelper<DynInteger>{.bits=bits_num, .data=DynInteger(static_cast<T>(tval))};
    }
    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("%" PRIi64 " [%sint_%d_d%d]", (int64_t)value, (std::is_signed<T>::value) ? "" : "u",
            std::numeric_limits<T>::digits, DYN_BITS);
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
    // check that T is bitis type
    //T values[ARRAY_SIZE];
    std::array<T, ARRAY_SIZE> values;

    FixedArray() : values() {}
    explicit FixedArray(const std::array<T, ARRAY_SIZE> &v) : values(v) {
        // for (std::size_t i = 0; i < ARRAY_SIZE; i++) { values[i] = v[i]; }
    }

    uint8_t serialize(BitisSerializer &ser) const {
        uint8_t num_bits = 0;
        for (std::size_t i=0; i < ARRAY_SIZE; i++) {
            num_bits += values[i].serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<FixedArray> deserialize(BitisDeserializer &des) {
        std::size_t num_bits = 0;
        //T values[ARRAY_SIZE];
        std::array<T, ARRAY_SIZE> values{};

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
            values[i].print(0);
            if (i != ARRAY_SIZE-1) printf(", ");
        }
        printf("]{%zu},\n", ARRAY_SIZE);
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

    DynArray() : values() {}
    explicit DynArray(const std::vector<T> &v) : values(v) {}

    uint8_t serialize(BitisSerializer &ser) const {
        uint8_t num_bits = 0;
        auto data_size = DynInteger<uint32_t, DYN_BITS>(values.size());
        num_bits += data_size.serialize(ser);
        for (std::size_t i=0; i < values.size(); i++) {
            // num_bits += ser.add_data(values[i].serialize(ser));
            num_bits += values[i].serialize(ser);
        }
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<DynArray> deserialize(BitisDeserializer &des) {
        std::size_t num_bits = 0;

        auto rv_size = DynInteger<uint32_t, DYN_BITS>::deserialize(des);
        num_bits += rv_size.bits;

        auto tvalues = std::vector<T>(rv_size.data.value);
        for (std::size_t i=0; i < rv_size.data.value; i++) {
            auto dr = T::deserialize(des);
            tvalues[i]= dr.data;
            num_bits += dr.bits;
        }
        return bitis_helper::BitiaDeserializerHelper<DynArray>{.bits=num_bits, .data=DynArray(tvalues)};
    }
    void print(int16_t indent=0) {
        // ReSharper disable once CppCStyleCast
        printf("[");
        for (std::size_t i=0; i < values.size(); i++) {
            values[i].print(0);
            if (i != values.size()-1) printf(", ");
        }
        printf("]{dyn:%zu},\n", values.size());
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
enum FixPrecisionMinMaxEnum {Ok, Underflow, Overflow};

template <uint8_t BITS, int64_t MIN_IVALUE, int64_t MAX_IVALUE>
struct FixPrecisionMinMax {
    static_assert(BITS > 2);

    FixPrecisionMinMax() : value(), state(Ok) { }
    explicit FixPrecisionMinMax(const double &v) : value(0) { // NOLINT(*-pro-type-member-init)
        set(v);
    }

    void set(const double &v) {
        value = 0;
        state = Ok;

        if (v < MIN_IVALUE) {
            state = Underflow;
        }
        else if (v > MAX_IVALUE) {
            state = Overflow;
        }
        else {
            value = v;
        }
    }
    double get_value() const { return value; }
    FixPrecisionMinMaxEnum get_state() const { return state; }

    uint8_t serialize(BitisSerializer &ser) const {
        uint8_t num_bits = 0;
        auto max_value = calc_bitmask(BITS);

        uint64_t v;
        if (state == Underflow) { v = 0; }
        else if (state == Overflow) { v = max_value; }
        else {
            v = lround((value - ((double)MIN_IVALUE))
                / ((double)(MAX_IVALUE - MIN_IVALUE))
                * (double)(max_value - 2)) + 1;
        }
        auto t = IntgralWithGivenBitSize<uint64_t, BITS>{v};
        printf("t: "); t.print(-1); printf("\n");
        num_bits += t.serialize(ser);
        return num_bits;
    }
    static bitis_helper::BitiaDeserializerHelper<FixPrecisionMinMax> deserialize(BitisDeserializer &des) {
        auto max_value = calc_bitmask(BITS);

        auto d = FixPrecisionMinMax(0.0);
        auto data_u64 = IntgralWithGivenBitSize<uint64_t, BITS>::deserialize(des);
        if(data_u64.data.value == 0) {
            d.state = Underflow;
        } else if(data_u64.data.value == max_value) {
            d.state = Overflow;
        } else {
            d.state = Ok;
            d.value = (((double)data_u64.data.value-1.) / (static_cast<double>(max_value-2)) *
                ((double)(MAX_IVALUE-MIN_IVALUE))) + (double)MIN_IVALUE;
        }
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
        printf(" [FP%ld-%ld]", MIN_IVALUE, MAX_IVALUE);
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
struct Binary : DynArray<uint8_t, 8> {
    Binary() : DynArray<uint8_t, 8>() {}
    explicit Binary(const std::vector<uint8_t> &v) : DynArray(v) {}
    // void print(int16_t indent=0) {
    //     // ReSharper disable once CppCStyleCast
    //     printf("{%zu}[", ARRAY_SIZE);
    //     for (std::size_t i=0; i < ARRAY_SIZE; i++) {
    //         values[i].print(0);
    //         if (i != ARRAY_SIZE-1) printf(", ");
    //     }
    //     printf("],\n");
    // }
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
    struct EnumeratedListImpl<Collector<ETs...>, ETN<T, CIDX>, TT, TTs...> {
        typedef typename EnumeratedListImpl<Collector<ETs..., ETN<T, CIDX>>,
            ETN<TT, CIDX+1>, TTs...>::type type;
    };
    template<typename T, uint32_t CIDX, typename ... ETs>
    struct EnumeratedListImpl<Collector<ETs...>, ETN<T, CIDX>> {
        typedef Collector<ETs..., ETN<T, CIDX>> type;
    };

    template<typename ... > struct EnumeratedList;
    template<typename T, typename ... Ts>
    struct EnumeratedList<T, Ts ...> {
        typedef typename EnumeratedListImpl<Collector<>, ETN<T, 0>, Ts...>::type type;
    };

    template<typename ... > struct EnumeratedListCollector;
    template<typename T, typename ... Ts>
    struct EnumeratedListCollector<Collector<T, Ts ...>> {
        typedef typename EnumeratedListImpl<Collector<>, ETN<T, 0>, Ts...>::type type;
    };

}

// ***************************************************************
namespace bitis_enum_helper {
    using namespace bitis_helper;

    template<typename ...> struct EnumID_Impl;
    template<typename SEL_ENUM, typename T, typename ... Ts>
    struct EnumID_Impl<SEL_ENUM, Collector<T, Ts ...>> {
        static size_t get_id() {
            return EnumID_Impl<SEL_ENUM, Collector<Ts ...>>::get_id();
        }
    };
    template<typename T, typename ... Ts>
    struct EnumID_Impl<typename T::type, Collector<T, Ts ...>> {
        static size_t get_id() { return T::id::value; }
    };
    template<typename SEL_ENUM>
    struct EnumID_Impl<SEL_ENUM, Collector<>> {
        // ReSharper disable once CppStaticAssertFailure
        static_assert(false, "Enum unknown");
        static size_t get_id() { return 0; }
    };
    template<typename ...> struct EnumName_Impl;
    template<typename T, typename ... Ts>
    struct EnumName_Impl<Collector<T, Ts ...>> {
        static const char *get_name(uint32_t enum_id) {
            if (enum_id == T::id::value) {
                return T::type::name;
            }
            return EnumName_Impl<Collector<Ts ...>>::get_name(enum_id);
        }
    };
    template<>
    struct EnumName_Impl<Collector<>> {
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

struct EnumBase { };

// template<typename ...> struct BitisEnum;
template<typename ES_COLLECTOR, uint8_t DYN_BITS>
struct BitisEnum : DynInteger<uint32_t, DYN_BITS> {
    typedef typename bitis_helper::EnumeratedListCollector<ES_COLLECTOR>::type EnumeratedEnumCollector;

    BitisEnum() : DynInteger<uint32_t, DYN_BITS>() {}

private:
    explicit BitisEnum(uint32_t enum_id) : DynInteger<uint32_t, DYN_BITS>(enum_id) {}

public:
    template<typename SET_ENUM>
    static BitisEnum create_enum() {
        auto id_in = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
        return BitisEnum(id_in);
    }

    template<typename SET_ENUM>
    void set_enum() {
        this->value = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
    }
    template<typename SET_ENUM>
    bool is_enum() {
        auto id_in = bitis_enum_helper::get_id<SET_ENUM, EnumeratedEnumCollector>();
        return this->value == id_in;
    }

    void print(int16_t indent=0) {
        auto v = bitis_enum_helper::get_name<EnumeratedEnumCollector>(this->value);
        // ReSharper disable once CppCStyleCast
        printf("%s (id:%" PRIu32 ") [enum_%u]", v, this->value, DYN_BITS);
    }
    BitisEnum &operator=(const BitisEnum &other) {
        this->value = other.value;
        return *this;
    }
    BitisEnum &operator=(const DynInteger<uint32_t, DYN_BITS> &other) {
        this->value = other.value;
        return *this;
    }
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
    template<typename ... Ts>
    struct MessageTE<Collector<Ts ...>> : MessageAttribute<Ts> ... { };

    template<typename ... Ts>
    struct MessageT : MessageTE<typename EnumeratedList<Ts...>::type> {

    };


    template<typename ...> struct MessageT_Impl;
    template<typename Msg_STRUCT, typename T, typename ... Ts>
    struct MessageT_Impl<Msg_STRUCT, Collector<T, Ts ...>> {
        size_t serialize(Msg_STRUCT *d, BitisSerializer &ser) {
            auto dd = static_cast<MessageAttribute<T>*>(d);

            std::size_t r = dd->value.serialize(ser);

            MessageT_Impl<Msg_STRUCT, Collector<Ts ...>> inner;
            return r + inner.serialize(d, ser);
        }
        size_t deserialize(Msg_STRUCT *d, BitisDeserializer &des) {
            auto dd = static_cast<MessageAttribute<T>*>(d);
            bitis_helper::BitiaDeserializerHelper<typename MessageAttribute<T>::type> r =
                MessageAttribute<T>::type::deserialize(des);
            dd->value = r.data;
            MessageT_Impl<Msg_STRUCT, Collector<Ts ...>> inner;
            return r.bits + inner.deserialize(d, des);
        }

        void print(Msg_STRUCT *d, int16_t indent, const char **attr_names) {
            print_indent(indent);
            printf("%s: ", attr_names[MessageAttribute<T>::id::value]);

            auto dd = static_cast<MessageAttribute<T>*>(d);
            dd->value.print(indent);
            if (indent >= 0) printf(",\n");
            else printf(", ");

            MessageT_Impl<Msg_STRUCT, Collector<Ts ...>> inner;
            return inner.print(d, indent, attr_names);
        }
    };
    template<typename Msg_STRUCT>
    struct MessageT_Impl<Msg_STRUCT, Collector<>> {
        // ReSharper disable once CppMemberFunctionMayBeStatic
        size_t serialize(Msg_STRUCT *d, BitisSerializer &ser) { return 0; }
        size_t deserialize(Msg_STRUCT *d, BitisDeserializer &) { return 0; }
        void print(Msg_STRUCT *, int16_t , const char **) { }
    };

    template<typename ...> struct MessageT_ImplStart;
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
        char buffer[sizeof(Msg)];
        // ReSharper disable once CppCStyleCast
        auto *dt = (typename Msg::MsgT*) buffer;

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

    // ***

    // template<typename STATELIST, typename STATE>
    // struct StateListElement {
    //     STATE state;
    // };
    //
    // template<typename ...> struct StateListImpl;
    // template<typename STATELIST, typename ... STATEs>
    // struct StateListImpl<STATELIST, Collector<STATEs ...>> : StateListElement<STATELIST, STATEs> ... { };


}

namespace oneof_helper {
    using namespace bitis_helper;

    template<typename ...> struct OneofAttribute;
    template<typename T, uint32_t IDX> struct OneofAttribute<ETN<T, IDX>> {
        typedef std::integral_constant<uint32_t, IDX> id;
        typedef T type;
    };

    template<typename ...> struct OneOfTE;
    template<typename ... Ts>
    struct OneOfTE<Collector<Ts ...>> : OneofAttribute<Ts> ... { };

    template<typename ... Ts>
    struct OneOfT : OneOfTE<typename EnumeratedList<Ts...>::type> { };

    template<typename ...> struct OneOfT_Impl;
    template<typename OOT_STRUCT, typename OOT, typename T, typename ... Ts>
    struct OneOfT_Impl<OOT_STRUCT, OOT, Collector<T, Ts ...>> {
        size_t serialize(OOT_STRUCT *d, BitisSerializer &ser) {
            // ReSharper disable once CppCStyleCast
            auto active_oneof = (uint32_t) d->oo_selector;
            if (active_oneof == T::id::value) {
                // ReSharper disable once CppCStyleCast
                const std::size_t r = ((typename T::type *)&(d->oo))->serialize(ser);
                return r;
            }
            OneOfT_Impl<OOT_STRUCT, OOT, Collector<Ts ...>> inner;
            return inner.serialize(d, ser);
        }
        void oneof_print(OOT_STRUCT *d, int16_t indent, const char **oo_enums) {
            // ReSharper disable once CppCStyleCast
            auto active_oneof = (uint32_t) d->oo_selector;
            if (active_oneof == T::id::value) {
                // ReSharper disable once CppCStyleCast
                printf("[%s] -> ", oo_enums[T::id::value]);
                // ReSharper disable once CppCStyleCast
                ((typename T::type *)&(d->oo))->print(indent);
                // if (indent >= 0) printf(",\n");
                // else printf(", ");
                return;
            }
            OneOfT_Impl<OOT_STRUCT, OOT, Collector<Ts ...>> inner;
            return inner.oneof_print(d, indent, oo_enums);
        }
    };
    template<typename OOT_STRUCT, typename OOT>
    struct OneOfT_Impl<OOT_STRUCT, OOT, Collector<>> {
        // ReSharper disable once CppMemberFunctionMayBeStatic
        size_t serialize(OOT_STRUCT *d, BitisSerializer &ser) { return 0; }
        void oneof_print(OOT_STRUCT *, int16_t , const char **) { }
    };

    template<typename ...> struct OneOfT_ImplStart;
    template<typename OOT_STRUCT, typename ... Ts>
    struct OneOfT_ImplStart<OOT_STRUCT, OneOfT<Ts ...>> {
        size_t serialize(OOT_STRUCT *d, BitisSerializer &ser) {
            OneOfT_Impl<OOT_STRUCT, OneOfT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            return inner.serialize(d, ser);
        }
        void oneof_print(OOT_STRUCT *d, int16_t indent, const char **oo_enums) {
            OneOfT_Impl<OOT_STRUCT, OneOfT<Ts ...>, typename EnumeratedList<Ts ...>::type> inner;
            inner.oneof_print(d, indent, oo_enums);
        }
    };

    template<typename OOTS>
    size_t oneof_serialize(OOTS *d, BitisSerializer &ser) {
        OneOfT_ImplStart<OOTS, typename OOTS::OneOfT> inner;
        return inner.serialize(d, ser);
    }
    template<typename OOTS>
    void oneof_print(OOTS *d, int16_t indent, const char **oo_enums) {
        // ReSharper disable once CppCStyleCast
        OneOfT_ImplStart<OOTS, typename OOTS::OneOfT> inner;
        inner.oneof_print(d, indent, oo_enums);
    }

}



