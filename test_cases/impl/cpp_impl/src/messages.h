#include "bitis_lib.h"
#include <optional>



// ****** OO_MsgKvoo_Value *****


struct OO_MsgKvoo_Value  {
    struct OO_StrVal {
        static constexpr auto name = "StrVal"; typedef BitisString<4> OOType; };
    struct OO_NumVal {
        static constexpr auto name = "NumVal"; typedef BitisFloatingPoint<double> OOType; };
    struct OO_BoolVal {
        static constexpr auto name = "BoolVal"; typedef BitisBool OOType; };
    struct OO_IntVal {
        static constexpr auto name = "IntVal"; typedef DynInteger<int32_t, 7> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_StrVal, 
        OO_NumVal, 
        OO_BoolVal, 
        OO_IntVal
    >, OO_BoolVal, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<
        OO_StrVal::OOType, OO_NumVal::OOType, OO_BoolVal::OOType, OO_IntVal::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_MsgKvoo_Value() : oo_selector(), oo_value() {}

    template<typename OOT>
    OO_MsgKvoo_Value set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(v);
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

    std::size_t serialize(BitisSerializer &ser) {
        return oneof_helper::oneof_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<OO_MsgKvoo_Value> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_MsgKvoo_Value>(des);
    }

    void print(const int16_t indent=0) {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_MsgKvoo_Value &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_MsgKvoo_Value &other) const { return is_equal(other); }
    bool operator!=(const OO_MsgKvoo_Value &other) const { return !is_equal(other); }
};


// ****** MsgKVSimple *****


struct MsgKVSimple {
    typedef BitisString<4> Key_T;
    typedef BitisString<4> Value_T;

    typedef message_helper::MessageT<
        Key_T, Value_T
    > MsgT;

    Key_T key;
    Value_T value;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgKVSimple> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVSimple>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgKVSimple{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVSimple &other) const {
        return key==other.key && value==other.value;
   }
    bool operator==(const MsgKVSimple &other) const { return is_equal(other); }
    bool operator!=(const MsgKVSimple &other) const { return !is_equal(other); }
};
const char *MsgKVSimple::msg_attr[] = {"key", "value"};

// ****** MsgKVOO *****


struct MsgKVOO {
    typedef BitisString<4> Key_T;
    typedef OO_MsgKvoo_Value Value_T;

    typedef message_helper::MessageT<
        Key_T, Value_T
    > MsgT;

    Key_T key;
    Value_T value;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgKVOO> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVOO>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgKVOO{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVOO &other) const {
        return key==other.key && value==other.value;
   }
    bool operator==(const MsgKVOO &other) const { return is_equal(other); }
    bool operator!=(const MsgKVOO &other) const { return !is_equal(other); }
};
const char *MsgKVOO::msg_attr[] = {"key", "value"};

// ****** MsgKVMapSimple *****


struct MsgKVMapSimple {
    typedef DynArray<MsgKVSimple,4> Entries_T;

    typedef message_helper::MessageT<
        Entries_T
    > MsgT;

    Entries_T entries;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgKVMapSimple> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVMapSimple>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgKVMapSimple{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVMapSimple &other) const {
        return entries==other.entries;
   }
    bool operator==(const MsgKVMapSimple &other) const { return is_equal(other); }
    bool operator!=(const MsgKVMapSimple &other) const { return !is_equal(other); }
};
const char *MsgKVMapSimple::msg_attr[] = {"entries"};

// ****** MsgKVMapOO *****


struct MsgKVMapOO {
    typedef DynArray<MsgKVOO,2> Entries_T;

    typedef message_helper::MessageT<
        Entries_T
    > MsgT;

    Entries_T entries;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgKVMapOO> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVMapOO>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgKVMapOO{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVMapOO &other) const {
        return entries==other.entries;
   }
    bool operator==(const MsgKVMapOO &other) const { return is_equal(other); }
    bool operator!=(const MsgKVMapOO &other) const { return !is_equal(other); }
};
const char *MsgKVMapOO::msg_attr[] = {"entries"};