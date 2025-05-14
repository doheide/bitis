#include "bitis_lib.h"
#include <optional>

// Enums
namespace NumbersEnum {
    ENUM_INSTANCE(One);
    ENUM_INSTANCE(Two);
    ENUM_INSTANCE(Three);
    ENUM_INSTANCE(Four);
}
typedef BitisEnum<bitis_helper::Collector<
    NumbersEnum::One, 
    NumbersEnum::Two, 
    NumbersEnum::Three, 
    NumbersEnum::Four
>, 4> Numbers;

// Enums for oneof
struct OO_ParamTestWithOo_Action  {
    struct OO_Inner {
        static constexpr auto name = "Inner"; typedef Inner Type; };
    struct OO_Val {
        static constexpr auto name = "Val"; typedef IntgralWithGivenBitSize<int8_t, 3> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Inner::OOType, 
        OO_Val::OOType
    >, 4> OOEnum;

    OOEnum oo_selector;
//    union {
//        char _base;
//        OO_Inner::OOType inner;
//
//        OO_Val::OOType val;
//
//    } oo;
    oneof_helper::UnionT<
        OO_Inner::OOType, OO_Val::OOType
    > oo_value;


    OO_ParamTestWithOo_Action() : oo_selector(), oo{(0)} {}

    template<typename OOT>
    OO_ParamTestWithOo_Action set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(v);
        return *this;
    }
    template<typename OOT>
    OOT *get_oo() {
        static_assert(oneof_helper::ContainsType<OOT, OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>()) 
            return oo_value.get<OOT>();
        return nullptr;
    }

    std::size_t serialize(BitisSerializer &ser) const {
        return oneof_helper::oneof_serialize(this, ser);
    }
    void print(const int16_t indent=0) {
        printf("Oneof ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent, oo_enums);
    }
};



// Messages
struct Inner {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 3> Val_T;
    typedef BitisOptional<BitisBool> OptBool_T;

    typedef message_helper::MessageT<
        Val_T, OptBool_T
    > MsgT;

    Val_T val;
    OptBool_T opt_bool;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<Inner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Inner>(des);
    }

    void print(int16_t indent=0) {
        printf("Inner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }
};
const char *Inner::msg_attr[] = {"val", "opt_bool"};

struct InnerWithEnum {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 3> Val_T;
    typedef BitisOptional<BitisBool> OptBool_T;
    typedef Numbers Num_T;

    typedef message_helper::MessageT<
        Val_T, OptBool_T, Num_T
    > MsgT;

    Val_T val;
    OptBool_T opt_bool;
    Num_T num;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<InnerWithEnum> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<InnerWithEnum>(des);
    }

    void print(int16_t indent=0) {
        printf("InnerWithEnum{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }
};
const char *InnerWithEnum::msg_attr[] = {"val", "opt_bool", "num"};

struct ParamTestWithInner {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 4> Param1_T;
    typedef Inner Inner_T;

    typedef message_helper::MessageT<
        Param1_T, Inner_T
    > MsgT;

    Param1_T param_1;
    Inner_T inner;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<ParamTestWithInner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<ParamTestWithInner>(des);
    }

    void print(int16_t indent=0) {
        printf("ParamTestWithInner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }
};
const char *ParamTestWithInner::msg_attr[] = {"param_1", "inner"};

struct ParamTestWithOo {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 4> Param1_T;
    typedef OO_ParamTestWithOo_Action Action_T;

    typedef message_helper::MessageT<
        Param1_T, Action_T
    > MsgT;

    Param1_T param_1;
    Action_T action;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<ParamTestWithOo> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<ParamTestWithOo>(des);
    }

    void print(int16_t indent=0) {
        printf("ParamTestWithOo{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }
};
const char *ParamTestWithOo::msg_attr[] = {"param_1", "action"};


