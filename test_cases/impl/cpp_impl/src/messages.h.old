#include "bitis_lib.h"
#include <optional>



// ****** Inner *****


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

    bool is_equal(const Inner &other) const {
        return val==other.val && opt_bool==other.opt_bool;
   }
    bool operator==(const Inner &other) const { return is_equal(other); }
    bool operator!=(const Inner &other) const { return !is_equal(other); }
};
const char *Inner::msg_attr[] = {"val", "opt_bool"};

// ****** Numbers *****
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



// ****** InnerWithEnum *****


struct InnerWithEnum {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 3> Val_T;
    typedef Numbers Num_T;
    typedef BitisOptional<BitisBool> OptBool_T;

    typedef message_helper::MessageT<
        Val_T, Num_T, OptBool_T
    > MsgT;

    Val_T val;
    Num_T num;
    OptBool_T opt_bool;

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

    bool is_equal(const InnerWithEnum &other) const {
        return val==other.val && num==other.num && opt_bool==other.opt_bool;
   }
    bool operator==(const InnerWithEnum &other) const { return is_equal(other); }
    bool operator!=(const InnerWithEnum &other) const { return !is_equal(other); }
};
const char *InnerWithEnum::msg_attr[] = {"val", "num", "opt_bool"};

// ****** ParamTestWithInner *****


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

    bool is_equal(const ParamTestWithInner &other) const {
        return param_1==other.param_1 && inner==other.inner;
   }
    bool operator==(const ParamTestWithInner &other) const { return is_equal(other); }
    bool operator!=(const ParamTestWithInner &other) const { return !is_equal(other); }
};
const char *ParamTestWithInner::msg_attr[] = {"param_1", "inner"};

// ****** OO_ParamTestWithOo_Action *****


struct OO_ParamTestWithOo_Action  {
    struct OO_Inner {
        static constexpr auto name = "Inner"; typedef Inner OOType; };
    struct OO_Val {
        static constexpr auto name = "Val"; typedef IntgralWithGivenBitSize<int8_t, 3> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Inner, 
        OO_Val
    >, 4> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<
        OO_Inner::OOType, OO_Val::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_ParamTestWithOo_Action() : oo_selector(), oo_value() {}

    template<typename OOT>
    OO_ParamTestWithOo_Action set_oo(typename OOT::OOType v) {
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
    static bitis_helper::BitiaDeserializerHelper<OO_ParamTestWithOo_Action> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_ParamTestWithOo_Action>(des);
    }

    void print(const int16_t indent=0) {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_ParamTestWithOo_Action &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_ParamTestWithOo_Action &other) const { return is_equal(other); }
    bool operator!=(const OO_ParamTestWithOo_Action &other) const { return !is_equal(other); }
};


// ****** ParamTestWithOo *****


struct ParamTestWithOo {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 4> Param1_T;
    typedef OO_ParamTestWithOo_Action Action_T;
    typedef Numbers Num_T;

    typedef message_helper::MessageT<
        Param1_T, Action_T, Num_T
    > MsgT;

    Param1_T param_1;
    Action_T action;
    Num_T num;

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

    bool is_equal(const ParamTestWithOo &other) const {
        return param_1==other.param_1 && action==other.action && num==other.num;
   }
    bool operator==(const ParamTestWithOo &other) const { return is_equal(other); }
    bool operator!=(const ParamTestWithOo &other) const { return !is_equal(other); }
};
const char *ParamTestWithOo::msg_attr[] = {"param_1", "action", "num"};