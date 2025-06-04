#include "bitis_lib.h"
#include <optional>



// ****** MsgSimpleTestBase *****


struct MsgSimpleTestBase {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;
    typedef BitisBool Param2_T;
    typedef IntgralWithGivenBitSize<int8_t, 5> Param3_T;

    typedef message_helper::MessageT<
        Param1_T, Param2_T, Param3_T
    > MsgT;

    Param1_T param_1;
    Param2_T param_2;
    Param3_T param_3;

    std::size_t serialize(BitisSerializer &ser) const {
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
        return param_1==other.param_1 && param_2==other.param_2 && param_3==other.param_3;
   }
    bool operator==(const MsgSimpleTestBase &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleTestBase &other) const { return !is_equal(other); }
};
const char *MsgSimpleTestBase::msg_attr[] = {"param_1", "param_2", "param_3"};

// ****** MsgSimpleBaseThreeInt *****


struct MsgSimpleBaseThreeInt {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;
    typedef IntgralWithGivenBitSize<uint8_t, 6> Param2_T;
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param3_T;

    typedef message_helper::MessageT<
        Param1_T, Param2_T, Param3_T
    > MsgT;

    Param1_T param_1;
    Param2_T param_2;
    Param3_T param_3;

    std::size_t serialize(BitisSerializer &ser) const {
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
        return param_1==other.param_1 && param_2==other.param_2 && param_3==other.param_3;
   }
    bool operator==(const MsgSimpleBaseThreeInt &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleBaseThreeInt &other) const { return !is_equal(other); }
};
const char *MsgSimpleBaseThreeInt::msg_attr[] = {"param_1", "param_2", "param_3"};

// ****** MsgSimpleTestFP *****


struct MsgSimpleTestFp {
    static const char *msg_attr[];
    typedef BitisBool Param1_T;
    typedef FixPrecisionMinMax<10, -1, 1> Fp_T;

    typedef message_helper::MessageT<
        Param1_T, Fp_T
    > MsgT;

    Param1_T param_1;
    Fp_T fp;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgSimpleTestFp> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgSimpleTestFp>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgSimpleTestFp{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgSimpleTestFp &other) const {
        return param_1==other.param_1 && fp==other.fp;
   }
    bool operator==(const MsgSimpleTestFp &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleTestFp &other) const { return !is_equal(other); }
};
const char *MsgSimpleTestFp::msg_attr[] = {"param_1", "fp"};

// ****** MsgSimpleOpt *****


struct MsgSimpleOpt {
    static const char *msg_attr[];
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

    std::size_t serialize(BitisSerializer &ser) const {
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
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;

    typedef message_helper::MessageT<
        Param1_T
    > MsgT;

    Param1_T param_1;

    std::size_t serialize(BitisSerializer &ser) const {
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