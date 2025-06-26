#include "bitis_lib.h"
#include <optional>

//#define EXPECTED_BITIS_VERSION "0.8.1"
//#if EXPECTED_BITIS_VERSION != BITIS_CPP_LIB_VERSION
//#error "Unexpected bitis library version"
//#endif



// ****** MsgSimpleTestFP *****


struct MsgSimpleTestFP {
    typedef BitisBool Param1_T;
    typedef FixPrecisionMinMax<10, -1, 1> Fp_T;

    typedef message_helper::MessageT<
        Param1_T, Fp_T
    > MsgT;

    Param1_T param_1;
    Fp_T fp;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgSimpleTestFP> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgSimpleTestFP>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgSimpleTestFP{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgSimpleTestFP &other) const {
        return param_1==other.param_1 && fp==other.fp;
   }
    bool operator==(const MsgSimpleTestFP &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleTestFP &other) const { return !is_equal(other); }
};
const char *MsgSimpleTestFP::msg_attr[] = {"param_1", "fp"};

// ****** MsgSimpleBaseThreeInt *****


struct MsgSimpleBaseThreeInt {
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;
    typedef IntgralWithGivenBitSize<uint16_t, 6> Param2_T;
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param3_T;
    typedef DynInteger<uint16_t, 6, 4> Param4_T;

    typedef message_helper::MessageT<
        Param1_T, Param2_T, Param3_T, Param4_T
    > MsgT;

    Param1_T param_1;
    Param2_T param_2;
    Param3_T param_3;
    Param4_T param_4;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgSimpleBaseThreeInt> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgSimpleBaseThreeInt>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgSimpleBaseThreeInt{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgSimpleBaseThreeInt &other) const {
        return param_1==other.param_1 && param_2==other.param_2 && param_3==other.param_3 && param_4==other.param_4;
   }
    bool operator==(const MsgSimpleBaseThreeInt &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleBaseThreeInt &other) const { return !is_equal(other); }
};
const char *MsgSimpleBaseThreeInt::msg_attr[] = {"param_1", "param_2", "param_3", "param_4"};

// ****** MsgSimpleTestBase *****


struct MsgSimpleTestBase {
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;
    typedef BitisBool Param2_T;
    typedef IntgralWithGivenBitSize<int16_t, 5> Param3_T;
    typedef BitisAString<4> Name_T;

    typedef message_helper::MessageT<
        Param1_T, Param2_T, Param3_T, Name_T
    > MsgT;

    Param1_T param_1;
    Param2_T param_2;
    Param3_T param_3;
    Name_T name;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgSimpleTestBase> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgSimpleTestBase>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgSimpleTestBase{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgSimpleTestBase &other) const {
        return param_1==other.param_1 && param_2==other.param_2 && param_3==other.param_3 && name==other.name;
   }
    bool operator==(const MsgSimpleTestBase &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleTestBase &other) const { return !is_equal(other); }
};
const char *MsgSimpleTestBase::msg_attr[] = {"param_1", "param_2", "param_3", "name"};

// ****** MsgSimpleOpt *****


struct MsgSimpleOpt {
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;
    typedef BitisOptional<BitisBool> Param2_T;
    typedef BitisOptional<IntgralWithGivenBitSize<uint16_t, 11>> Param3_T;
    typedef BitisOptional<FixPrecisionMinMax<10, -1, 1>> Param4_T;

    typedef message_helper::MessageT<
        Param1_T, Param2_T, Param3_T, Param4_T
    > MsgT;

    Param1_T param_1;
    Param2_T param_2;
    Param3_T param_3;
    Param4_T param_4;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgSimpleOpt> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgSimpleOpt>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgSimpleOpt{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgSimpleOpt &other) const {
        return param_1==other.param_1 && param_2==other.param_2 && param_3==other.param_3 && param_4==other.param_4;
   }
    bool operator==(const MsgSimpleOpt &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleOpt &other) const { return !is_equal(other); }
};
const char *MsgSimpleOpt::msg_attr[] = {"param_1", "param_2", "param_3", "param_4"};

// ****** MsgSimpleBaseOneInt *****


struct MsgSimpleBaseOneInt {
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;

    typedef message_helper::MessageT<
        Param1_T
    > MsgT;

    Param1_T param_1;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgSimpleBaseOneInt> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgSimpleBaseOneInt>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgSimpleBaseOneInt{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgSimpleBaseOneInt &other) const {
        return param_1==other.param_1;
   }
    bool operator==(const MsgSimpleBaseOneInt &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleBaseOneInt &other) const { return !is_equal(other); }
};
const char *MsgSimpleBaseOneInt::msg_attr[] = {"param_1"};