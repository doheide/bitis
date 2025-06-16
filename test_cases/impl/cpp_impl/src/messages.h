#include "bitis_lib.h"
#include <optional>

//#define EXPECTED_BITIS_VERSION "0.6.12"
//#if EXPECTED_BITIS_VERSION != BITIS_CPP_LIB_VERSION
//#error "Unexpected bitis library version"
//#endif



// ****** MsgWithDynInt *****


struct MsgWithDynInt {
    typedef DynInteger<uint8_t, 4, 3> Val_T;
    typedef DynInteger<int8_t, 4, 3> SignedVal_T;

    typedef message_helper::MessageT<
        Val_T, SignedVal_T
    > MsgT;

    Val_T val;
    SignedVal_T signed_val;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgWithDynInt> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgWithDynInt>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgWithDynInt{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgWithDynInt &other) const {
        return val==other.val && signed_val==other.signed_val;
   }
    bool operator==(const MsgWithDynInt &other) const { return is_equal(other); }
    bool operator!=(const MsgWithDynInt &other) const { return !is_equal(other); }
};
const char *MsgWithDynInt::msg_attr[] = {"val", "signed_val"};