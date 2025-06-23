#include "bitis_lib.h"
#include <optional>

//#define EXPECTED_BITIS_VERSION "0.7.1"
//#if EXPECTED_BITIS_VERSION != BITIS_CPP_LIB_VERSION
//#error "Unexpected bitis library version"
//#endif



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



// ****** Inner *****


struct Inner {
    typedef IntgralWithGivenBitSize<int16_t, 3> Val2_T;

    typedef message_helper::MessageT<
        Val2_T
    > MsgT;

    Val2_T val2;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
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
        return val2==other.val2;
   }
    bool operator==(const Inner &other) const { return is_equal(other); }
    bool operator!=(const Inner &other) const { return !is_equal(other); }
};
const char *Inner::msg_attr[] = {"val2"};

// ****** SensorSource *****
namespace SensorSourceEnum {
    ENUM_INSTANCE(TemperaturSensor);
    ENUM_INSTANCE(MovementSensor);
}

typedef BitisEnum<bitis_helper::Collector<
    SensorSourceEnum::TemperaturSensor, 
    SensorSourceEnum::MovementSensor
>, SensorSourceEnum::TemperaturSensor, 3> SensorSource;



// ****** MsgFixedBaseArray *****


struct MsgFixedBaseArray {
    typedef SensorSource Param1_T;
    typedef FixedArray<IntgralWithGivenBitSize<uint16_t, 3>,3> Val1_T;
    typedef FixedArray<IntgralWithGivenBitSize<int16_t, 3>,3> Val2_T;
    typedef FixedArray<BitisBool,3> Val3_T;
    typedef FixedArray<DynInteger<int16_t, 8, 3>,3> Val4_T;
    typedef FixedArray<BitisFloatingPoint<double>,3> Val5_T;
    typedef FixedArray<FixPrecisionMinMax<10, -2, 3>,3> Val6_T;
    typedef FixedArray<SensorSource,3> Val7_T;
    typedef FixedArray<Inner,3> Val8_T;

    typedef message_helper::MessageT<
        Param1_T, Val1_T, Val2_T, Val3_T, Val4_T, Val5_T, Val6_T, Val7_T, Val8_T
    > MsgT;

    Param1_T param_1;
    Val1_T val1;
    Val2_T val2;
    Val3_T val3;
    Val4_T val4;
    Val5_T val5;
    Val6_T val6;
    Val7_T val7;
    Val8_T val8;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgFixedBaseArray> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgFixedBaseArray>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgFixedBaseArray{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgFixedBaseArray &other) const {
        return param_1==other.param_1 && val1==other.val1 && val2==other.val2 && val3==other.val3 && val4==other.val4 && val5==other.val5 && val6==other.val6 && val7==other.val7 && val8==other.val8;
   }
    bool operator==(const MsgFixedBaseArray &other) const { return is_equal(other); }
    bool operator!=(const MsgFixedBaseArray &other) const { return !is_equal(other); }
};
const char *MsgFixedBaseArray::msg_attr[] = {"param_1", "val1", "val2", "val3", "val4", "val5", "val6", "val7", "val8"};