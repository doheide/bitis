#pragma once

#include "bitis_lib.h"
#include <optional>
#include <cstdint>

// ****** schema stamp *****
// Compiler version and schema hash (first 8 bytes of SHA-256 over the canonical schema, see bitis
// docs/WIRE_FORMAT.md). Always available as bitis_schema_all_types::BITIS_VERSION / ::BITIS_SCHEMA_HASH;
// the unqualified names refer to the first generated header included in a translation unit.
namespace bitis_schema_all_types {
    constexpr const char* BITIS_VERSION = "{BITIS_VERSION}";
    constexpr uint64_t BITIS_SCHEMA_HASH = 0xc10a57d77c346314ULL;
}
#ifndef BITIS_SCHEMA_STAMP_DEFINED
#define BITIS_SCHEMA_STAMP_DEFINED
constexpr const char* BITIS_VERSION = bitis_schema_all_types::BITIS_VERSION;
constexpr uint64_t BITIS_SCHEMA_HASH = bitis_schema_all_types::BITIS_SCHEMA_HASH;
#endif



// ****** Color *****
namespace ColorEnum {
    ENUM_INSTANCE(Red);
    ENUM_INSTANCE(Green);
    ENUM_INSTANCE(BlueLight);
}

typedef BitisEnum<bitis_helper::Collector<
    ColorEnum::Red, 
    ColorEnum::Green, 
    ColorEnum::BlueLight
>, ColorEnum::Green, 2> Color;



// ****** Inner *****


struct Inner {
    typedef IntgralWithGivenBitSize<uint16_t, 3> A_T;

    typedef message_helper::MessageT<
        A_T
    > MsgT;

    A_T a;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Inner>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Inner>(); }
    static bitis_helper::BitiaDeserializerHelper<Inner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Inner>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[1] = {"a"};
    void print(int16_t indent=0) const {
        printf("Inner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Inner &other) const {
        return a==other.a;}
    bool operator==(const Inner &other) const { return is_equal(other); }
    bool operator!=(const Inner &other) const { return !is_equal(other); }
};

// ****** Empty *****


struct Empty {

    typedef message_helper::MessageT<
        
    > MsgT;


    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Empty>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Empty>(); }
    static bitis_helper::BitiaDeserializerHelper<Empty> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Empty>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[0 + 1] = {};
    void print(int16_t indent=0) const {
        printf("Empty{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Empty &other) const {
        (void) other; return true;}
    bool operator==(const Empty &other) const { return is_equal(other); }
    bool operator!=(const Empty &other) const { return !is_equal(other); }
};

// ****** OO_AllTypes_Choice *****


struct OO_AllTypes_Choice  {
    struct OO_Flag {
        BITIS_NAME("Flag") typedef BitisBool OOType; };
    struct OO_Number {
        BITIS_NAME("Number") typedef IntgralWithGivenBitSize<uint16_t, 12> OOType; };
    struct OO_InnerVal {
        BITIS_NAME("InnerVal") typedef Inner OOType; };
    struct OO_ColorVal {
        BITIS_NAME("ColorVal") typedef Color OOType; };
    struct OO_Text {
        BITIS_NAME("Text") typedef BitisAString<3> OOType; };
    struct OO_Nothing {
        BITIS_NAME("Nothing") typedef Empty OOType; };
    struct OO_Raw {
        BITIS_NAME("Raw") typedef Binary<4> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Flag, 
        OO_Number, 
        OO_InnerVal, 
        OO_ColorVal, 
        OO_Text, 
        OO_Nothing, 
        OO_Raw
    >, OO_Flag, 3> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Flag::OOType, OO_Number::OOType, OO_InnerVal::OOType, OO_ColorVal::OOType, OO_Text::OOType, OO_Nothing::OOType, OO_Raw::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_AllTypes_Choice() : oo_selector() {}

    template<typename OOT>
    OO_AllTypes_Choice set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
        return nullptr;
    }
    template<typename OOT>
    bool is_oo_value() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return true;
        return false;
    }

    std::size_t serialize(BitisSerializer &ser) const {
        return oneof_helper::oneof_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_AllTypes_Choice>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_AllTypes_Choice>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_AllTypes_Choice> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_AllTypes_Choice>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_AllTypes_Choice &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_AllTypes_Choice &other) const { return is_equal(other); }
    bool operator!=(const OO_AllTypes_Choice &other) const { return !is_equal(other); }
};


// ****** AllTypes *****


struct AllTypes {
    typedef BitisBool B_T;
    typedef IntgralWithGivenBitSize<uint16_t, 1> U1_T;
    typedef IntgralWithGivenBitSize<uint32_t, 17> U17_T;
    typedef IntgralWithGivenBitSize<uint64_t, 64> U64V_T;
    typedef IntgralWithGivenBitSize<int16_t, 5> I5_T;
    typedef IntgralWithGivenBitSize<int64_t, 33> I33_T;
    typedef DynInteger<uint32_t, 32, 4> Ud_T;
    typedef DynInteger<int16_t, 16, 3> Id_T;
    typedef BitisFloatingPoint<float> F_T;
    typedef BitisFloatingPoint<double> D_T;
    typedef FixPrecisionMinMax<10, -2, 3> Fp_T;
    typedef FixPrecisionMinMax<34, -90, 90> Lat_T;
    typedef BitisAString<4> S_T;
    typedef Binary<6> Data_T;
    typedef Color Color_T;
    typedef Inner Inner_T;
    typedef Empty Empty_T;
    typedef BitisOptional<IntgralWithGivenBitSize<uint16_t, 7>> OptU_T;
    typedef BitisOptional<Inner> OptInner_T;
    typedef BitisOptional<Empty> OptEmpty_T;
    typedef FixedArray<BitisBool,3> RfB_T;
    typedef FixedArray<Inner,2> RfInner_T;
    typedef DynArray<IntgralWithGivenBitSize<uint16_t, 4>,2> RdU_T;
    typedef DynArray<Color,3> RdColor_T;
    typedef DynArray<Inner,4> RdInner_T;
    typedef OO_AllTypes_Choice Choice_T;

    typedef message_helper::MessageT<
        B_T, U1_T, U17_T, U64V_T, I5_T, I33_T, Ud_T, Id_T, F_T, D_T, Fp_T, Lat_T, S_T, Data_T, Color_T, Inner_T, Empty_T, OptU_T, OptInner_T, OptEmpty_T, RfB_T, RfInner_T, RdU_T, RdColor_T, RdInner_T, Choice_T
    > MsgT;

    B_T b;
    U1_T u1;
    U17_T u17;
    U64V_T u64v;
    I5_T i5;
    I33_T i33;
    Ud_T ud;
    Id_T id;
    F_T f;
    D_T d;
    Fp_T fp;
    Lat_T lat;
    S_T s;
    Data_T data;
    Color_T color;
    Inner_T inner;
    Empty_T empty;
    OptU_T opt_u;
    OptInner_T opt_inner;
    OptEmpty_T opt_empty;
    RfB_T rf_b;
    RfInner_T rf_inner;
    RdU_T rd_u;
    RdColor_T rd_color;
    RdInner_T rd_inner;
    Choice_T choice;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<AllTypes>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<AllTypes>(); }
    static bitis_helper::BitiaDeserializerHelper<AllTypes> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<AllTypes>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[26] = {"b", "u1", "u17", "u64v", "i5", "i33", "ud", "id", "f", "d", "fp", "lat", "s", "data", "color", "inner", "empty", "opt_u", "opt_inner", "opt_empty", "rf_b", "rf_inner", "rd_u", "rd_color", "rd_inner", "choice"};
    void print(int16_t indent=0) const {
        printf("AllTypes{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const AllTypes &other) const {
        return b==other.b && u1==other.u1 && u17==other.u17 && u64v==other.u64v && i5==other.i5 && i33==other.i33 && ud==other.ud && id==other.id && f==other.f && d==other.d && fp==other.fp && lat==other.lat && s==other.s && data==other.data && color==other.color && inner==other.inner && empty==other.empty && opt_u==other.opt_u && opt_inner==other.opt_inner && opt_empty==other.opt_empty && rf_b==other.rf_b && rf_inner==other.rf_inner && rd_u==other.rd_u && rd_color==other.rd_color && rd_inner==other.rd_inner && choice==other.choice;}
    bool operator==(const AllTypes &other) const { return is_equal(other); }
    bool operator!=(const AllTypes &other) const { return !is_equal(other); }
};