#pragma once

#include "bitis_lib.h"
#include <optional>
#include <cstdint>

// ****** schema stamp *****
// Compiler version and schema hash (first 8 bytes of SHA-256 over the canonical schema, see bitis
// docs/WIRE_FORMAT.md). Always available as bitis_schema_test_bounds::BITIS_VERSION / ::BITIS_SCHEMA_HASH;
// the unqualified names refer to the first generated header included in a translation unit.
namespace bitis_schema_test_bounds {
    constexpr const char* BITIS_VERSION = "0.10.5";
    constexpr uint64_t BITIS_SCHEMA_HASH = 0xf7f9348cd3819475ULL;
}
#ifndef BITIS_SCHEMA_STAMP_DEFINED
#define BITIS_SCHEMA_STAMP_DEFINED
constexpr const char* BITIS_VERSION = bitis_schema_test_bounds::BITIS_VERSION;
constexpr uint64_t BITIS_SCHEMA_HASH = bitis_schema_test_bounds::BITIS_SCHEMA_HASH;
#endif



// ****** BColor *****
namespace BColorEnum {
    ENUM_INSTANCE(Red);
    ENUM_INSTANCE(Green);
    ENUM_INSTANCE(Blue);
}

typedef BitisEnum<bitis_helper::Collector<
    BColorEnum::Red, 
    BColorEnum::Green, 
    BColorEnum::Blue
>, BColorEnum::Red, 2> BColor;



// ****** BLeaf *****


struct BLeaf {
    typedef IntgralWithGivenBitSize<uint16_t, 4> V_T;

    typedef message_helper::MessageT<
        V_T
    > MsgT;

    V_T v;
    const char * str_v = "v";
    const char *msg_attr[1] = {str_v};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<BLeaf>(); }
    static bitis_helper::BitiaDeserializerHelper<BLeaf> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<BLeaf>(des);
    }

    void print(int16_t indent=0) const {
        printf("BLeaf{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const BLeaf &other) const {
        return v==other.v;}
    bool operator==(const BLeaf &other) const { return is_equal(other); }
    bool operator!=(const BLeaf &other) const { return !is_equal(other); }
};
//const char *BLeaf::msg_attr[] = {"v"};

// ****** OO_Bounded_Choice *****


struct OO_Bounded_Choice  {
    struct OO_Text {
        static constexpr auto name = "Text"; typedef BoundedAString<3, 4> OOType; };
    struct OO_Raw {
        static constexpr auto name = "Raw"; typedef BoundedBinary<4, 2> OOType; };
    struct OO_Leaf {
        static constexpr auto name = "Leaf"; typedef BLeaf OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Text, 
        OO_Raw, 
        OO_Leaf
    >, OO_Text, 1> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Text::OOType, OO_Raw::OOType, OO_Leaf::OOType
    > T_OOValue;
    T_OOValue oo_value;
    //uint8_t *oo_value_ptr;

    OO_Bounded_Choice() : oo_selector() /*, oo_value_ptr(nullptr)*/ {}
    //~OO_Bounded_Choice() { delete [] oo_value_ptr; }

    template<typename OOT>
    OO_Bounded_Choice set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;

        //delete [] oo_value_ptr;
        //oo_value_ptr = new uint8_t[sizeof(typename OOT::OOType)];
        //memcpy(oo_value_ptr, &v, sizeof(v));
        //return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
            //return (typename OOT::OOType*) oo_value_ptr;
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
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Bounded_Choice>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Bounded_Choice> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Bounded_Choice>(des);
    }

    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_Bounded_Choice &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Bounded_Choice &other) const { return is_equal(other); }
    bool operator!=(const OO_Bounded_Choice &other) const { return !is_equal(other); }
};


// ****** Bounded *****


struct Bounded {
    typedef BoundedAString<4, 8> Name_T;
    typedef BoundedBinary<3, 4> Blob_T;
    typedef BoundedDynArray<BoundedAString<4, 5>,2,3> Tags_T;
    typedef BitisOptional<BoundedAString<2, 6>> Note_T;
    typedef FixedArray<BoundedBinary<2, 3>,2> Pair_T;
    typedef BoundedDynArray<BLeaf,1,2> Leaves_T;
    typedef BoundedDynArray<BColor,2,4> Colors_T;
    typedef OO_Bounded_Choice Choice_T;
    typedef BitisBool Tail_T;

    typedef message_helper::MessageT<
        Name_T, Blob_T, Tags_T, Note_T, Pair_T, Leaves_T, Colors_T, Choice_T, Tail_T
    > MsgT;

    Name_T name;
    Blob_T blob;
    Tags_T tags;
    Note_T note;
    Pair_T pair;
    Leaves_T leaves;
    Colors_T colors;
    Choice_T choice;
    Tail_T tail;
    const char * str_name = "name";
    const char * str_blob = "blob";
    const char * str_tags = "tags";
    const char * str_note = "note";
    const char * str_pair = "pair";
    const char * str_leaves = "leaves";
    const char * str_colors = "colors";
    const char * str_choice = "choice";
    const char * str_tail = "tail";
    const char *msg_attr[9] = {str_name, str_blob, str_tags, str_note, str_pair, str_leaves, str_colors, str_choice, str_tail};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Bounded>(); }
    static bitis_helper::BitiaDeserializerHelper<Bounded> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Bounded>(des);
    }

    void print(int16_t indent=0) const {
        printf("Bounded{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const Bounded &other) const {
        return name==other.name && blob==other.blob && tags==other.tags && note==other.note && pair==other.pair && leaves==other.leaves && colors==other.colors && choice==other.choice && tail==other.tail;}
    bool operator==(const Bounded &other) const { return is_equal(other); }
    bool operator!=(const Bounded &other) const { return !is_equal(other); }
};
//const char *Bounded::msg_attr[] = {"name", "blob", "tags", "note", "pair", "leaves", "colors", "choice", "tail"};