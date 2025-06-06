#include "bitis_lib.h"
#include <optional>



// ****** ExampleEnum *****
namespace ExampleEnumEnum {
    ENUM_INSTANCE(E1);
    ENUM_INSTANCE(E2);
    ENUM_INSTANCE(E3);
    ENUM_INSTANCE(E4);
    ENUM_INSTANCE(E5);
    ENUM_INSTANCE(E6);
    ENUM_INSTANCE(E7);
    ENUM_INSTANCE(E8);
    ENUM_INSTANCE(E9);
}

typedef BitisEnum<bitis_helper::Collector<
    ExampleEnumEnum::E1, 
    ExampleEnumEnum::E2, 
    ExampleEnumEnum::E3, 
    ExampleEnumEnum::E4, 
    ExampleEnumEnum::E5, 
    ExampleEnumEnum::E6, 
    ExampleEnumEnum::E7, 
    ExampleEnumEnum::E8, 
    ExampleEnumEnum::E9
>, ExampleEnumEnum::E3, 2> ExampleEnum;



// ****** SensorSource *****
namespace SensorSourceEnum {
    ENUM_INSTANCE(TemperaturSensor);
    ENUM_INSTANCE(MovementSensor);
}

typedef BitisEnum<bitis_helper::Collector<
    SensorSourceEnum::TemperaturSensor, 
    SensorSourceEnum::MovementSensor
>, SensorSourceEnum::TemperaturSensor, 3> SensorSource;



// ****** MsgEnumOpt *****


struct MsgEnumOpt {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 3> Val_T;
    typedef SensorSource Param1_T;
    typedef BitisOptional<ExampleEnum> Param2_T;

    typedef message_helper::MessageT<
        Val_T, Param1_T, Param2_T
    > MsgT;

    Val_T val;
    Param1_T param_1;
    Param2_T param_2;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgEnumOpt> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgEnumOpt>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgEnumOpt{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgEnumOpt &other) const {
        return val==other.val && param_1==other.param_1 && param_2==other.param_2;
   }
    bool operator==(const MsgEnumOpt &other) const { return is_equal(other); }
    bool operator!=(const MsgEnumOpt &other) const { return !is_equal(other); }
};
const char *MsgEnumOpt::msg_attr[] = {"val", "param_1", "param_2"};

// ****** MsgWithInner *****


struct MsgWithInner {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 3> Val_T;
    typedef MsgEnumOpt Imsg_T;

    typedef message_helper::MessageT<
        Val_T, Imsg_T
    > MsgT;

    Val_T val;
    Imsg_T imsg;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgWithInner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgWithInner>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgWithInner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgWithInner &other) const {
        return val==other.val && imsg==other.imsg;
   }
    bool operator==(const MsgWithInner &other) const { return is_equal(other); }
    bool operator!=(const MsgWithInner &other) const { return !is_equal(other); }
};
const char *MsgWithInner::msg_attr[] = {"val", "imsg"};

// ****** MsgWithTwoInner *****


struct MsgWithTwoInner {
    static const char *msg_attr[];
    typedef IntgralWithGivenBitSize<uint8_t, 3> Val_T;
    typedef MsgWithInner Imsg_T;
    typedef BitisOptional<MsgEnumOpt> Oimsg_T;

    typedef message_helper::MessageT<
        Val_T, Imsg_T, Oimsg_T
    > MsgT;

    Val_T val;
    Imsg_T imsg;
    Oimsg_T oimsg;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgWithTwoInner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgWithTwoInner>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgWithTwoInner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgWithTwoInner &other) const {
        return val==other.val && imsg==other.imsg && oimsg==other.oimsg;
   }
    bool operator==(const MsgWithTwoInner &other) const { return is_equal(other); }
    bool operator!=(const MsgWithTwoInner &other) const { return !is_equal(other); }
};
const char *MsgWithTwoInner::msg_attr[] = {"val", "imsg", "oimsg"};