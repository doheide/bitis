#pragma once

#include "bitis_lib.h"
#include <optional>
#include <cstdint>

// ****** schema stamp *****
// Compiler version and schema hash (first 8 bytes of SHA-256 over the canonical schema, see bitis
// docs/WIRE_FORMAT.md). Always available as bitis_schema_nested::BITIS_VERSION / ::BITIS_SCHEMA_HASH;
// the unqualified names refer to the first generated header included in a translation unit.
namespace bitis_schema_nested {
    constexpr const char* BITIS_VERSION = "{BITIS_VERSION}";
    constexpr uint64_t BITIS_SCHEMA_HASH = 0xf9d3f02b9ff943e5ULL;
}
#ifndef BITIS_SCHEMA_STAMP_DEFINED
#define BITIS_SCHEMA_STAMP_DEFINED
constexpr const char* BITIS_VERSION = bitis_schema_nested::BITIS_VERSION;
constexpr uint64_t BITIS_SCHEMA_HASH = bitis_schema_nested::BITIS_SCHEMA_HASH;
#endif



// ****** E *****
namespace EEnum {
    ENUM_INSTANCE(A);
    ENUM_INSTANCE(B);
}

typedef BitisEnum<bitis_helper::Collector<
    EEnum::A, 
    EEnum::B
>, EEnum::A, 1> E;



// ****** Leaf *****


struct Leaf {
    typedef IntgralWithGivenBitSize<uint16_t, 2> V_T;

    typedef message_helper::MessageT<
        V_T
    > MsgT;

    V_T v;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Leaf>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Leaf>(); }
    static bitis_helper::BitiaDeserializerHelper<Leaf> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Leaf>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[1] = {"v"};
    void print(int16_t indent=0) const {
        printf("Leaf{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Leaf &other) const {
        return v==other.v;}
    bool operator==(const Leaf &other) const { return is_equal(other); }
    bool operator!=(const Leaf &other) const { return !is_equal(other); }
};

// ****** Middle *****


struct Middle {
    typedef DynArray<Leaf,2> Leaves_T;
    typedef BitisOptional<E> E_T;

    typedef message_helper::MessageT<
        Leaves_T, E_T
    > MsgT;

    Leaves_T leaves;
    E_T e;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Middle>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Middle>(); }
    static bitis_helper::BitiaDeserializerHelper<Middle> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Middle>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"leaves", "e"};
    void print(int16_t indent=0) const {
        printf("Middle{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Middle &other) const {
        return leaves==other.leaves && e==other.e;}
    bool operator==(const Middle &other) const { return is_equal(other); }
    bool operator!=(const Middle &other) const { return !is_equal(other); }
};

// ****** OO_Outer_Inner *****


struct OO_Outer_Inner  {
    struct OO_X {
        BITIS_NAME("X") typedef BitisBool OOType; };
    struct OO_E {
        BITIS_NAME("E") typedef E OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_X, 
        OO_E
    >, OO_X, 1> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_X::OOType, OO_E::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_Outer_Inner() : oo_selector() {}

    template<typename OOT>
    OO_Outer_Inner set_oo(typename OOT::OOType v) {
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
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Outer_Inner>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_Outer_Inner>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Outer_Inner> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Outer_Inner>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_Outer_Inner &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Outer_Inner &other) const { return is_equal(other); }
    bool operator!=(const OO_Outer_Inner &other) const { return !is_equal(other); }
};


// ****** OO_Top_Outer *****


struct OO_Top_Outer  {
    struct OO_Leaf {
        BITIS_NAME("Leaf") typedef Leaf OOType; };
    struct OO_Inner {
        BITIS_NAME("Inner") typedef OO_Outer_Inner OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Leaf, 
        OO_Inner
    >, OO_Leaf, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Leaf::OOType, OO_Inner::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_Top_Outer() : oo_selector() {}

    template<typename OOT>
    OO_Top_Outer set_oo(typename OOT::OOType v) {
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
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Top_Outer>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_Top_Outer>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Top_Outer> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Top_Outer>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_Top_Outer &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Top_Outer &other) const { return is_equal(other); }
    bool operator!=(const OO_Top_Outer &other) const { return !is_equal(other); }
};


// ****** Top *****


struct Top {
    typedef Middle M_T;
    typedef OO_Top_Outer Outer_T;

    typedef message_helper::MessageT<
        M_T, Outer_T
    > MsgT;

    M_T m;
    Outer_T outer;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Top>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Top>(); }
    static bitis_helper::BitiaDeserializerHelper<Top> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Top>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"m", "outer"};
    void print(int16_t indent=0) const {
        printf("Top{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Top &other) const {
        return m==other.m && outer==other.outer;}
    bool operator==(const Top &other) const { return is_equal(other); }
    bool operator!=(const Top &other) const { return !is_equal(other); }
};