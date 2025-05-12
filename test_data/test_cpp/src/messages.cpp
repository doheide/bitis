#include "bitis_lib.h"
#include <optional>

// Enums
namespace NumbersEnum {
    ENUM_INSTANCE(One);
    ENUM_INSTANCE(Two);
    ENUM_INSTANCE(Three);
    ENUM_INSTANCE(Four);
}

// Enums for oneof
struct OO_ParamTestWithInner_Action  {
    static const char *oo_enums[];
    enum OOEnum {
        Inner, 
        Val
    };
    typedef std::integral_constant<uint8_t, 4> SelectorBits;
    typedef oneof_helper::OneOfT<
        Inner, 
        VarWithGivenBitSize<i8, 3>
    > OneOfT;
    OOEnum oo_selector;
    union {
        char _base;
        Inner inner;
        VarWithGivenBitSize<i8, 3> val;
    } oo;

    explicit OOEnum_Val(const OOEnum _oo_selector) : oo_selector(_oo_selector), oo{(0)} {}
    void set_inner(const Inner inner) {
        oo.inner = inner;
        oo_selector = Inner;
    }
    Inner *get_inner() {
        if (oo_selector == Inner) { return &oo.inner; }
        return nullptr;
    }
    OOEnum_Val create_inner(const Inner v) {
        auto oo = OOEnum_Val(OOEnum_OoParamTestWithInnerAction::OOEnum::Inner);
        oo.set_inner(v);
        // ReSharper disable once CppSomeObjectMembersMightNotBeInitialized
        return oo;
    };
    void set_val(const VarWithGivenBitSize<i8, 3> val) {
        oo.val = val;
        oo_selector = Val;
    }
    VarWithGivenBitSize<i8, 3> *get_val() {
        if (oo_selector == Val) { return &oo.val; }
        return nullptr;
    }
    OOEnum_Val create_val(const VarWithGivenBitSize<i8, 3> v) {
        auto oo = OOEnum_Val(OOEnum_OoParamTestWithInnerAction::OOEnum::Val);
        oo.set_val(v);
        // ReSharper disable once CppSomeObjectMembersMightNotBeInitialized
        return oo;
    };

    std::size_t serialize(BitisSerializer &ser) const {
        return oneof_helper::oneof_serialize(this, ser);
    }
    void print(int16_t indent=0) {
        printf("Oneof ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent, oo_enums);
    }
};
const char *OOEnum_Val::oo_enums[] = {"inner", "val"};



// Messages
struct Inner {
    static const char *msg_attr[];

    typedef message_helper::MessageT<
        IntgralWithGivenBitSize<uint8_t, 3>,
        Numbers,
    > MsgT;

    IntgralWithGivenBitSize<uint8_t, 3> val;
    Numbers num;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgA> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgA>(des);
    }

    void print(int16_t indent=0) {
        printf("Inner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }
};
const char *MsgA::msg_attr[] = {"val", "num"};

struct ParamTestWithInner {
    static const char *msg_attr[];

    typedef message_helper::MessageT<
        IntgralWithGivenBitSize<uint8_t, 4>,
        BitisOptional<bool>,
        OO_ParamTestWithInner_Action,
    > MsgT;

    IntgralWithGivenBitSize<uint8_t, 4> param_1;
    BitisOptional<bool> param_2;
    OO_ParamTestWithInner_Action action;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgA> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgA>(des);
    }

    void print(int16_t indent=0) {
        printf("ParamTestWithInner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }
};
const char *MsgA::msg_attr[] = {"param_1", "param_2", "action"};



#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct Inner {
  pub val: IntgralWithGivenBitSize<uint8_t, 3>,
  pub num: Numbers,
}
#[derive(BiserdiMsg, Debug, Clone, PartialEq)]
#[allow(nonstandard_style)]
pub struct ParamTestWithInner {
  pub param_1: IntgralWithGivenBitSize<uint8_t, 4>,
  pub param_2: Option<bool>,
  pub action: OO_ParamTestWithInner_Action,
}
