#pragma once

#include "bitis_lib.h"
#include <optional>
#include <cstdint>

// ****** schema stamp *****
// Compiler version and schema hash (first 8 bytes of SHA-256 over the canonical schema, see bitis
// docs/WIRE_FORMAT.md). Always available as bitis_schema_docs::BITIS_VERSION / ::BITIS_SCHEMA_HASH;
// the unqualified names refer to the first generated header included in a translation unit.
namespace bitis_schema_docs {
    constexpr const char* BITIS_VERSION = "{BITIS_VERSION}";
    constexpr uint64_t BITIS_SCHEMA_HASH = 0x18fe2976a5287710ULL;
}
#ifndef BITIS_SCHEMA_STAMP_DEFINED
#define BITIS_SCHEMA_STAMP_DEFINED
constexpr const char* BITIS_VERSION = bitis_schema_docs::BITIS_VERSION;
constexpr uint64_t BITIS_SCHEMA_HASH = bitis_schema_docs::BITIS_SCHEMA_HASH;
#endif



// ****** Kind *****
/// Enum doc with special chars: <html> & "quotes" \ 'single'
/// second line
namespace KindEnum {
    /// first value
    ENUM_INSTANCE(KindA);
    /// second value
    ENUM_INSTANCE(KindB);
}

typedef BitisEnum<bitis_helper::Collector<
    KindEnum::KindA, 
    KindEnum::KindB
>, KindEnum::KindA, 2> Kind;



// ****** OO_Documented_Sel *****


struct OO_Documented_Sel  {
    struct OO_Flag {
        static constexpr auto name = "Flag"; typedef BitisBool OOType; };
    struct OO_Num {
        static constexpr auto name = "Num"; typedef IntgralWithGivenBitSize<uint16_t, 3> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Flag, 
        OO_Num
    >, OO_Flag, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Flag::OOType, OO_Num::OOType
    > T_OOValue;
    T_OOValue oo_value;
    //uint8_t *oo_value_ptr;

    OO_Documented_Sel() : oo_selector() /*, oo_value_ptr(nullptr)*/ {}
    //~OO_Documented_Sel() { delete [] oo_value_ptr; }

    template<typename OOT>
    OO_Documented_Sel set_oo(typename OOT::OOType v) {
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
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Documented_Sel>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Documented_Sel> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Documented_Sel>(des);
    }

    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_Documented_Sel &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Documented_Sel &other) const { return is_equal(other); }
    bool operator!=(const OO_Documented_Sel &other) const { return !is_equal(other); }
};


// ****** Documented *****


/// Message doc
struct Documented {
    typedef IntgralWithGivenBitSize<uint16_t, 4> Value_T;
    typedef Kind Kind_T;
    typedef OO_Documented_Sel Sel_T;

    typedef message_helper::MessageT<
        Value_T, Kind_T, Sel_T
    > MsgT;


    /// attribute doc
    Value_T value;

    /// multi
    /// line attribute doc
    Kind_T kind;

    /// oneof doc
    Sel_T sel;
    const char * str_value = "value";
    const char * str_kind = "kind";
    const char * str_sel = "sel";
    const char *msg_attr[3] = {str_value, str_kind, str_sel};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Documented>(); }
    static bitis_helper::BitiaDeserializerHelper<Documented> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Documented>(des);
    }

    void print(int16_t indent=0) const {
        printf("Documented{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const Documented &other) const {
        return value==other.value && kind==other.kind && sel==other.sel;}
    bool operator==(const Documented &other) const { return is_equal(other); }
    bool operator!=(const Documented &other) const { return !is_equal(other); }
};
//const char *Documented::msg_attr[] = {"value", "kind", "sel"};