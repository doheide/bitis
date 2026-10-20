#pragma once

#include "bitis_lib.h"
#include <optional>
#include <cstdint>

// ****** schema stamp *****
// Compiler version and schema hash (first 8 bytes of SHA-256 over the canonical schema, see bitis
// docs/WIRE_FORMAT.md). Always available as bitis_schema_bounds::BITIS_VERSION / ::BITIS_SCHEMA_HASH;
// the unqualified names refer to the first generated header included in a translation unit.
namespace bitis_schema_bounds {
    constexpr const char* BITIS_VERSION = "{BITIS_VERSION}";
    constexpr uint64_t BITIS_SCHEMA_HASH = 0x940ea6464ca7ba60ULL;
}
#ifndef BITIS_SCHEMA_STAMP_DEFINED
#define BITIS_SCHEMA_STAMP_DEFINED
constexpr const char* BITIS_VERSION = bitis_schema_bounds::BITIS_VERSION;
constexpr uint64_t BITIS_SCHEMA_HASH = bitis_schema_bounds::BITIS_SCHEMA_HASH;
#endif



// ****** Kind *****
namespace KindEnum {
    ENUM_INSTANCE(Small);
    ENUM_INSTANCE(Large);
}

typedef BitisEnum<bitis_helper::Collector<
    KindEnum::Small, 
    KindEnum::Large
>, KindEnum::Small, 2> Kind;



// ****** Point *****


struct Point {
    typedef IntgralWithGivenBitSize<uint16_t, 8> X_T;
    typedef IntgralWithGivenBitSize<uint16_t, 8> Y_T;

    typedef message_helper::MessageT<
        X_T, Y_T
    > MsgT;

    X_T x;
    Y_T y;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Point>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Point>(); }
    static bitis_helper::BitiaDeserializerHelper<Point> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Point>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"x", "y"};
    void print(int16_t indent=0) const {
        printf("Point{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Point &other) const {
        return x==other.x && y==other.y;}
    bool operator==(const Point &other) const { return is_equal(other); }
    bool operator!=(const Point &other) const { return !is_equal(other); }
};

// ****** OO_Bounded_Payload *****


struct OO_Bounded_Payload  {
    struct OO_Text {
        BITIS_NAME("Text") typedef BoundedAString<2, 5> OOType; };
    struct OO_Raw {
        BITIS_NAME("Raw") typedef BoundedAString<4, 6> OOType; };
    struct OO_P {
        BITIS_NAME("P") typedef Point OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Text, 
        OO_Raw, 
        OO_P
    >, OO_Text, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Text::OOType, OO_Raw::OOType, OO_P::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_Bounded_Payload() : oo_selector() {}

    template<typename OOT>
    OO_Bounded_Payload set_oo(typename OOT::OOType v) {
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
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Bounded_Payload>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_Bounded_Payload>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Bounded_Payload> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Bounded_Payload>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_Bounded_Payload &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Bounded_Payload &other) const { return is_equal(other); }
    bool operator!=(const OO_Bounded_Payload &other) const { return !is_equal(other); }
};


// ****** Bounded *****


/// Bounded strings, bytes and lists
struct Bounded {
    typedef BoundedAString<4, 16> Name_T;
    typedef BoundedBinary<3, 8> Blob_T;
    typedef BoundedDynArray<BoundedAString<4, 8>,2,4> Tags_T;
    typedef BitisOptional<BoundedAString<4, 32>> Note_T;
    typedef BoundedDynArray<Point,3,3> Points_T;
    typedef BoundedDynArray<Kind,1,2> Kinds_T;
    typedef FixedArray<BoundedBinary<2, 3>,2> Pair_T;
    typedef BitisAString<4> Unbounded_T;
    typedef OO_Bounded_Payload Payload_T;

    typedef message_helper::MessageT<
        Name_T, Blob_T, Tags_T, Note_T, Points_T, Kinds_T, Pair_T, Unbounded_T, Payload_T
    > MsgT;

    Name_T name;
    Blob_T blob;
    Tags_T tags;
    Note_T note;
    Points_T points;
    Kinds_T kinds;
    Pair_T pair;
    Unbounded_T unbounded;
    Payload_T payload;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Bounded>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Bounded>(); }
    static bitis_helper::BitiaDeserializerHelper<Bounded> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Bounded>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[9] = {"name", "blob", "tags", "note", "points", "kinds", "pair", "unbounded", "payload"};
    void print(int16_t indent=0) const {
        printf("Bounded{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Bounded &other) const {
        return name==other.name && blob==other.blob && tags==other.tags && note==other.note && points==other.points && kinds==other.kinds && pair==other.pair && unbounded==other.unbounded && payload==other.payload;}
    bool operator==(const Bounded &other) const { return is_equal(other); }
    bool operator!=(const Bounded &other) const { return !is_equal(other); }
};